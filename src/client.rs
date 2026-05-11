use log::debug;
use reqwest::{Client, Response, StatusCode};
use serde_json::Value;

use crate::error::{RedfishError, Result};
use crate::types::*;

/// Async Redfish client for BMC management.
pub struct RedfishClient {
    base_url: String,
    username: String,
    password: String,
    client: Client,
    session_token: Option<String>,
    session_uri: Option<String>,
}

impl RedfishClient {
    /// Create a new Redfish client.
    /// `host` can be IP or hostname. Uses HTTPS by default.
    pub fn new(host: &str, username: &str, password: &str) -> Result<Self> {
        let base_url = if host.starts_with("http") {
            host.trim_end_matches('/').to_string()
        } else {
            format!("https://{}", host.trim_end_matches('/'))
        };

        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            base_url,
            username: username.to_string(),
            password: password.to_string(),
            client,
            session_token: None,
            session_uri: None,
        })
    }

    /// Establish a Redfish session (POST to SessionService).
    pub async fn login(&mut self) -> Result<()> {
        let url = format!("{}/redfish/v1/SessionService/Sessions", self.base_url);
        let body = SessionCreate {
            user_name: self.username.clone(),
            password: self.password.clone(),
        };

        let resp = self.client.post(&url).json(&body).send().await?;
        if resp.status() == StatusCode::CREATED || resp.status() == StatusCode::OK {
            let token = resp
                .headers()
                .get("X-Auth-Token")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .ok_or(RedfishError::AuthFailed)?;
            let location = resp
                .headers()
                .get("Location")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            self.session_token = Some(token);
            self.session_uri = location;
            debug!("Session established");
            Ok(())
        } else {
            Err(RedfishError::AuthFailed)
        }
    }

    /// Close the Redfish session.
    pub async fn logout(&mut self) -> Result<()> {
        if let (Some(token), Some(uri)) = (&self.session_token, &self.session_uri) {
            let url = if uri.starts_with("http") {
                uri.clone()
            } else {
                format!("{}{}", self.base_url, uri)
            };
            let _ = self.client
                .delete(&url)
                .header("X-Auth-Token", token)
                .send()
                .await;
        }
        self.session_token = None;
        self.session_uri = None;
        Ok(())
    }

    /// GET a Redfish resource by path (e.g. "/redfish/v1/Systems").
    pub async fn get(&self, path: &str) -> Result<Value> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url, path)
        };
        let resp = self.auth_get(&url).await?;
        self.handle_response(resp).await
    }

    /// GET and deserialize into a typed struct.
    pub async fn get_as<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let val = self.get(path).await?;
        serde_json::from_value(val).map_err(|e| RedfishError::Parse(e.to_string()))
    }

    /// POST an action (e.g. Reset).
    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url, path)
        };
        let mut req = self.client.post(&url).json(body);
        if let Some(token) = &self.session_token {
            req = req.header("X-Auth-Token", token);
        } else {
            req = req.basic_auth(&self.username, Some(&self.password));
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    /// PATCH a resource (update properties).
    pub async fn patch(&self, path: &str, body: &Value) -> Result<Value> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url, path)
        };
        let mut req = self.client.patch(&url).json(body);
        if let Some(token) = &self.session_token {
            req = req.header("X-Auth-Token", token);
        } else {
            req = req.basic_auth(&self.username, Some(&self.password));
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    /// DELETE a resource.
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}{}", self.base_url, path)
        };
        let mut req = self.client.delete(&url);
        if let Some(token) = &self.session_token {
            req = req.header("X-Auth-Token", token);
        } else {
            req = req.basic_auth(&self.username, Some(&self.password));
        }
        let resp = req.send().await?;
        if resp.status().is_success() || resp.status() == StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            Err(RedfishError::Api { status, message: text })
        }
    }

    // --- High-level API ---

    /// Get Service Root.
    pub async fn get_service_root(&self) -> Result<ServiceRoot> {
        self.get_as("/redfish/v1/").await
    }

    /// List all computer systems.
    pub async fn list_systems(&self) -> Result<Collection> {
        self.get_as("/redfish/v1/Systems").await
    }

    /// Get a specific system by ID.
    pub async fn get_system(&self, id: &str) -> Result<ComputerSystem> {
        self.get_as(&format!("/redfish/v1/Systems/{}", id)).await
    }

    /// List all chassis.
    pub async fn list_chassis(&self) -> Result<Collection> {
        self.get_as("/redfish/v1/Chassis").await
    }

    /// Get a specific chassis by ID.
    pub async fn get_chassis(&self, id: &str) -> Result<Chassis> {
        self.get_as(&format!("/redfish/v1/Chassis/{}", id)).await
    }

    /// List all managers (BMCs).
    pub async fn list_managers(&self) -> Result<Collection> {
        self.get_as("/redfish/v1/Managers").await
    }

    /// Get a specific manager by ID.
    pub async fn get_manager(&self, id: &str) -> Result<Manager> {
        self.get_as(&format!("/redfish/v1/Managers/{}", id)).await
    }

    /// Get power info for a chassis.
    pub async fn get_power(&self, chassis_id: &str) -> Result<Power> {
        self.get_as(&format!("/redfish/v1/Chassis/{}/Power", chassis_id)).await
    }

    /// Get thermal info for a chassis.
    pub async fn get_thermal(&self, chassis_id: &str) -> Result<Thermal> {
        self.get_as(&format!("/redfish/v1/Chassis/{}/Thermal", chassis_id)).await
    }

    /// List processors for a system.
    pub async fn list_processors(&self, system_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Processors", system_id)).await
    }

    /// Get a specific processor.
    pub async fn get_processor(&self, system_id: &str, proc_id: &str) -> Result<Processor> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Processors/{}", system_id, proc_id)).await
    }

    /// List memory for a system.
    pub async fn list_memory(&self, system_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Memory", system_id)).await
    }

    /// Get a specific memory module.
    pub async fn get_memory(&self, system_id: &str, mem_id: &str) -> Result<Memory> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Memory/{}", system_id, mem_id)).await
    }

    /// List storage for a system.
    pub async fn list_storage(&self, system_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Storage", system_id)).await
    }

    /// Get a specific storage resource.
    pub async fn get_storage(&self, system_id: &str, storage_id: &str) -> Result<Storage> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Storage/{}", system_id, storage_id)).await
    }

    /// List ethernet interfaces for a system.
    pub async fn list_ethernet_interfaces(&self, system_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Systems/{}/EthernetInterfaces", system_id)).await
    }

    /// Get a specific ethernet interface.
    pub async fn get_ethernet_interface(&self, system_id: &str, iface_id: &str) -> Result<EthernetInterface> {
        self.get_as(&format!("/redfish/v1/Systems/{}/EthernetInterfaces/{}", system_id, iface_id)).await
    }

    /// Get account service.
    pub async fn get_account_service(&self) -> Result<AccountService> {
        self.get_as("/redfish/v1/AccountService").await
    }

    /// List user accounts.
    pub async fn list_accounts(&self) -> Result<Collection> {
        self.get_as("/redfish/v1/AccountService/Accounts").await
    }

    /// Get update service.
    pub async fn get_update_service(&self) -> Result<UpdateService> {
        self.get_as("/redfish/v1/UpdateService").await
    }

    /// Get event service.
    pub async fn get_event_service(&self) -> Result<EventService> {
        self.get_as("/redfish/v1/EventService").await
    }

    /// List log entries for a manager.
    pub async fn list_log_entries(&self, manager_id: &str, log_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Managers/{}/LogServices/{}/Entries", manager_id, log_id)).await
    }

    // --- Power Actions ---

    /// Reset/power control a system.
    /// reset_type: "On", "ForceOff", "GracefulShutdown", "GracefulRestart",
    ///             "ForceRestart", "Nmi", "ForceOn", "PushPowerButton", "PowerCycle"
    pub async fn reset_system(&self, system_id: &str, reset_type: &str) -> Result<Value> {
        let path = format!("/redfish/v1/Systems/{}/Actions/ComputerSystem.Reset", system_id);
        let body = serde_json::json!({ "ResetType": reset_type });
        self.post(&path, &body).await
    }

    /// Power on a system.
    pub async fn power_on(&self, system_id: &str) -> Result<Value> {
        self.reset_system(system_id, "On").await
    }

    /// Force power off a system.
    pub async fn power_off(&self, system_id: &str) -> Result<Value> {
        self.reset_system(system_id, "ForceOff").await
    }

    /// Graceful shutdown.
    pub async fn graceful_shutdown(&self, system_id: &str) -> Result<Value> {
        self.reset_system(system_id, "GracefulShutdown").await
    }

    /// Graceful restart.
    pub async fn graceful_restart(&self, system_id: &str) -> Result<Value> {
        self.reset_system(system_id, "GracefulRestart").await
    }

    /// Force restart.
    pub async fn force_restart(&self, system_id: &str) -> Result<Value> {
        self.reset_system(system_id, "ForceRestart").await
    }

    /// Power cycle.
    pub async fn power_cycle(&self, system_id: &str) -> Result<Value> {
        self.reset_system(system_id, "PowerCycle").await
    }

    // --- Boot Override ---

    /// Set boot source override.
    /// target: "None", "Pxe", "Cd", "Usb", "Hdd", "BiosSetup", "Diags"
    /// enabled: "Once", "Continuous", "Disabled"
    pub async fn set_boot_override(&self, system_id: &str, target: &str, enabled: Option<&str>) -> Result<Value> {
        let path = format!("/redfish/v1/Systems/{}", system_id);
        let body = serde_json::json!({
            "Boot": {
                "BootSourceOverrideTarget": target,
                "BootSourceOverrideEnabled": enabled.unwrap_or("Once")
            }
        });
        self.patch(&path, &body).await
    }

    /// Set next boot to PXE.
    pub async fn set_boot_pxe(&self, system_id: &str) -> Result<Value> {
        self.set_boot_override(system_id, "Pxe", Some("Once")).await
    }

    /// Set next boot to BIOS Setup.
    pub async fn set_boot_bios(&self, system_id: &str) -> Result<Value> {
        self.set_boot_override(system_id, "BiosSetup", Some("Once")).await
    }

    // --- Manager Actions ---

    /// Reset a manager (BMC).
    pub async fn reset_manager(&self, manager_id: &str, reset_type: &str) -> Result<Value> {
        let path = format!("/redfish/v1/Managers/{}/Actions/Manager.Reset", manager_id);
        let body = serde_json::json!({ "ResetType": reset_type });
        self.post(&path, &body).await
    }

    /// Clear a log service.
    pub async fn clear_log(&self, manager_id: &str, log_id: &str) -> Result<Value> {
        let path = format!("/redfish/v1/Managers/{}/LogServices/{}/Actions/LogService.ClearLog", manager_id, log_id);
        self.post(&path, &serde_json::json!({})).await
    }

    // --- Virtual Media ---

    /// List virtual media for a manager.
    pub async fn list_virtual_media(&self, manager_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Managers/{}/VirtualMedia", manager_id)).await
    }

    /// Get a virtual media resource.
    pub async fn get_virtual_media(&self, manager_id: &str, media_id: &str) -> Result<VirtualMedia> {
        self.get_as(&format!("/redfish/v1/Managers/{}/VirtualMedia/{}", manager_id, media_id)).await
    }

    /// Insert (mount) virtual media image.
    pub async fn insert_media(&self, manager_id: &str, media_id: &str, image_url: &str) -> Result<Value> {
        let path = format!("/redfish/v1/Managers/{}/VirtualMedia/{}/Actions/VirtualMedia.InsertMedia", manager_id, media_id);
        self.post(&path, &serde_json::json!({ "Image": image_url })).await
    }

    /// Eject (unmount) virtual media.
    pub async fn eject_media(&self, manager_id: &str, media_id: &str) -> Result<Value> {
        let path = format!("/redfish/v1/Managers/{}/VirtualMedia/{}/Actions/VirtualMedia.EjectMedia", manager_id, media_id);
        self.post(&path, &serde_json::json!({})).await
    }

    // --- BIOS ---

    /// Get BIOS attributes.
    pub async fn get_bios(&self, system_id: &str) -> Result<Bios> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Bios", system_id)).await
    }

    /// Get BIOS pending settings.
    pub async fn get_bios_settings(&self, system_id: &str) -> Result<Bios> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Bios/Settings", system_id)).await
    }

    /// Set BIOS attributes (applied on next boot).
    pub async fn set_bios_attributes(&self, system_id: &str, attributes: &Value) -> Result<Value> {
        let path = format!("/redfish/v1/Systems/{}/Bios/Settings", system_id);
        self.patch(&path, &serde_json::json!({ "Attributes": attributes })).await
    }

    // --- Secure Boot ---

    /// Get Secure Boot status.
    pub async fn get_secure_boot(&self, system_id: &str) -> Result<SecureBoot> {
        self.get_as(&format!("/redfish/v1/Systems/{}/SecureBoot", system_id)).await
    }

    /// Enable or disable Secure Boot.
    pub async fn set_secure_boot(&self, system_id: &str, enabled: bool) -> Result<Value> {
        let path = format!("/redfish/v1/Systems/{}/SecureBoot", system_id);
        self.patch(&path, &serde_json::json!({ "SecureBootEnable": enabled })).await
    }

    // --- Network Protocol ---

    /// Get manager network protocol settings.
    pub async fn get_network_protocol(&self, manager_id: &str) -> Result<NetworkProtocol> {
        self.get_as(&format!("/redfish/v1/Managers/{}/NetworkProtocol", manager_id)).await
    }

    /// Update network protocol settings (e.g. NTP servers).
    pub async fn set_network_protocol(&self, manager_id: &str, settings: &Value) -> Result<Value> {
        self.patch(&format!("/redfish/v1/Managers/{}/NetworkProtocol", manager_id), settings).await
    }

    // --- Serial Interfaces ---

    /// List serial interfaces for a manager.
    pub async fn list_serial_interfaces(&self, manager_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Managers/{}/SerialInterfaces", manager_id)).await
    }

    /// Get a serial interface.
    pub async fn get_serial_interface(&self, manager_id: &str, iface_id: &str) -> Result<SerialInterface> {
        self.get_as(&format!("/redfish/v1/Managers/{}/SerialInterfaces/{}", manager_id, iface_id)).await
    }

    // --- Volumes / RAID ---

    /// List volumes for a storage resource.
    pub async fn list_volumes(&self, system_id: &str, storage_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Storage/{}/Volumes", system_id, storage_id)).await
    }

    /// Get a specific volume.
    pub async fn get_volume(&self, system_id: &str, storage_id: &str, volume_id: &str) -> Result<Volume> {
        self.get_as(&format!("/redfish/v1/Systems/{}/Storage/{}/Volumes/{}", system_id, storage_id, volume_id)).await
    }

    /// Create a volume (RAID).
    pub async fn create_volume(&self, system_id: &str, storage_id: &str, body: &Value) -> Result<Value> {
        let path = format!("/redfish/v1/Systems/{}/Storage/{}/Volumes", system_id, storage_id);
        self.post(&path, body).await
    }

    /// Delete a volume.
    pub async fn delete_volume(&self, system_id: &str, storage_id: &str, volume_id: &str) -> Result<()> {
        self.delete(&format!("/redfish/v1/Systems/{}/Storage/{}/Volumes/{}", system_id, storage_id, volume_id)).await
    }

    // --- Drives ---

    /// Get a specific drive.
    pub async fn get_drive(&self, path: &str) -> Result<Drive> {
        self.get_as(path).await
    }

    // --- Certificates ---

    /// List certificates for a manager.
    pub async fn list_certificates(&self, manager_id: &str) -> Result<Collection> {
        self.get_as(&format!("/redfish/v1/Managers/{}/NetworkProtocol/HTTPS/Certificates", manager_id)).await
    }

    /// Get a certificate.
    pub async fn get_certificate(&self, path: &str) -> Result<Certificate> {
        self.get_as(path).await
    }

    /// Replace a certificate (POST new cert to collection or PATCH existing).
    pub async fn replace_certificate(&self, path: &str, cert_pem: &str, cert_type: &str) -> Result<Value> {
        self.post(path, &serde_json::json!({
            "CertificateString": cert_pem,
            "CertificateType": cert_type
        })).await
    }

    // --- Event Subscriptions ---

    /// List event subscriptions.
    pub async fn list_subscriptions(&self) -> Result<Collection> {
        self.get_as("/redfish/v1/EventService/Subscriptions").await
    }

    /// Create an event subscription.
    pub async fn create_subscription(&self, destination: &str, event_types: &[&str], context: &str) -> Result<Value> {
        let types: Vec<String> = event_types.iter().map(|s| s.to_string()).collect();
        self.post("/redfish/v1/EventService/Subscriptions", &serde_json::json!({
            "Destination": destination,
            "EventTypes": types,
            "Protocol": "Redfish",
            "Context": context
        })).await
    }

    /// Delete an event subscription.
    pub async fn delete_subscription(&self, subscription_id: &str) -> Result<()> {
        self.delete(&format!("/redfish/v1/EventService/Subscriptions/{}", subscription_id)).await
    }

    // --- Firmware Update ---

    /// List firmware inventory.
    pub async fn list_firmware_inventory(&self) -> Result<Collection> {
        self.get_as("/redfish/v1/UpdateService/FirmwareInventory").await
    }

    /// Get a firmware inventory item.
    pub async fn get_firmware_item(&self, item_id: &str) -> Result<SoftwareInventory> {
        self.get_as(&format!("/redfish/v1/UpdateService/FirmwareInventory/{}", item_id)).await
    }

    /// Simple firmware update via URI (BMC pulls the image).
    pub async fn simple_update(&self, image_uri: &str) -> Result<Value> {
        self.post("/redfish/v1/UpdateService/Actions/UpdateService.SimpleUpdate", &serde_json::json!({
            "ImageURI": image_uri
        })).await
    }

    // --- Tasks ---

    /// List tasks.
    pub async fn list_tasks(&self) -> Result<Collection> {
        self.get_as("/redfish/v1/TaskService/Tasks").await
    }

    /// Get a task by ID.
    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        self.get_as(&format!("/redfish/v1/TaskService/Tasks/{}", task_id)).await
    }

    /// Poll a task until completion (max wait in seconds).
    pub async fn wait_task(&self, task_id: &str, max_wait_secs: u64) -> Result<Task> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(max_wait_secs);
        loop {
            let task = self.get_task(task_id).await?;
            match task.task_state.as_deref() {
                Some("Completed") | Some("Exception") | Some("Killed") | Some("Cancelled") => {
                    return Ok(task);
                }
                _ => {}
            }
            if std::time::Instant::now() > deadline {
                return Ok(task);
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    // --- Pagination ---

    /// GET a collection and automatically follow @odata.nextLink to get all members.
    pub async fn get_all_members(&self, path: &str) -> Result<Vec<OdataLink>> {
        let mut all = Vec::new();
        let mut current_path = path.to_string();
        loop {
            let val = self.get(&current_path).await?;
            if let Some(members) = val.get("Members").and_then(|m| m.as_array()) {
                for m in members {
                    if let Some(id) = m.get("@odata.id").and_then(|v| v.as_str()) {
                        all.push(OdataLink { odata_id: id.to_string() });
                    }
                }
            }
            match val.get("Members@odata.nextLink").and_then(|v| v.as_str()) {
                Some(next) => current_path = next.to_string(),
                None => break,
            }
        }
        Ok(all)
    }

    // --- Internal helpers ---

    async fn auth_get(&self, url: &str) -> Result<Response> {
        let mut req = self.client.get(url);
        if let Some(token) = &self.session_token {
            req = req.header("X-Auth-Token", token);
        } else {
            req = req.basic_auth(&self.username, Some(&self.password));
        }
        Ok(req.send().await?)
    }

    async fn handle_response(&self, resp: Response) -> Result<Value> {
        let status = resp.status();
        if status.is_success() {
            let body = resp.text().await?;
            if body.is_empty() {
                Ok(Value::Null)
            } else {
                serde_json::from_str(&body).map_err(|e| RedfishError::Parse(e.to_string()))
            }
        } else if status == StatusCode::NOT_FOUND {
            Err(RedfishError::NotFound(resp.url().to_string()))
        } else if status == StatusCode::UNAUTHORIZED {
            Err(RedfishError::SessionExpired)
        } else {
            let code = status.as_u16();
            let text = resp.text().await.unwrap_or_default();
            Err(RedfishError::Api { status: code, message: text })
        }
    }
}
