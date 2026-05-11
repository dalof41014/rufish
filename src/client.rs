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
