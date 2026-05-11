use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Common Redfish resource status.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    pub state: Option<String>,
    pub health: Option<String>,
    pub health_rollup: Option<String>,
}

/// Service Root (/redfish/v1/)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceRoot {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub redfish_version: Option<String>,
    pub systems: Option<OdataLink>,
    pub chassis: Option<OdataLink>,
    pub managers: Option<OdataLink>,
    pub session_service: Option<OdataLink>,
    pub account_service: Option<OdataLink>,
    pub update_service: Option<OdataLink>,
    pub event_service: Option<OdataLink>,
}

/// OData link reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdataLink {
    #[serde(rename = "@odata.id")]
    pub odata_id: String,
}

/// Collection of resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Collection {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "Members@odata.count")]
    pub members_count: Option<u32>,
    #[serde(rename = "Members")]
    pub members: Option<Vec<OdataLink>>,
}

/// Computer System resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComputerSystem {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub uuid: Option<String>,
    pub host_name: Option<String>,
    pub power_state: Option<String>,
    pub bios_version: Option<String>,
    pub status: Option<Status>,
    pub boot: Option<Boot>,
    pub processor_summary: Option<ProcessorSummary>,
    pub memory_summary: Option<MemorySummary>,
    pub processors: Option<OdataLink>,
    pub memory: Option<OdataLink>,
    pub storage: Option<OdataLink>,
    pub ethernet_interfaces: Option<OdataLink>,
    pub network_interfaces: Option<OdataLink>,
    pub bios: Option<OdataLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Boot {
    pub boot_source_override_target: Option<String>,
    pub boot_source_override_enabled: Option<String>,
    pub boot_source_override_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProcessorSummary {
    pub count: Option<u32>,
    pub model: Option<String>,
    pub status: Option<Status>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MemorySummary {
    pub total_system_memory_gi_b: Option<f64>,
    pub status: Option<Status>,
}

/// Chassis resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Chassis {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub chassis_type: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub part_number: Option<String>,
    pub power_state: Option<String>,
    pub status: Option<Status>,
    pub thermal: Option<OdataLink>,
    pub power: Option<OdataLink>,
}

/// Manager (BMC) resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Manager {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub manager_type: Option<String>,
    pub firmware_version: Option<String>,
    pub status: Option<Status>,
    pub ethernet_interfaces: Option<OdataLink>,
    pub network_protocol: Option<OdataLink>,
    pub log_services: Option<OdataLink>,
}

/// Power resource (legacy schema).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Power {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub power_control: Option<Vec<PowerControl>>,
    pub power_supplies: Option<Vec<PowerSupply>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerControl {
    pub name: Option<String>,
    pub power_consumed_watts: Option<f64>,
    pub power_capacity_watts: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PowerSupply {
    pub name: Option<String>,
    pub power_capacity_watts: Option<f64>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub status: Option<Status>,
}

/// Thermal resource (legacy schema).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Thermal {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub temperatures: Option<Vec<Temperature>>,
    pub fans: Option<Vec<Fan>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Temperature {
    pub name: Option<String>,
    pub reading_celsius: Option<f64>,
    pub upper_threshold_critical: Option<f64>,
    pub upper_threshold_fatal: Option<f64>,
    pub status: Option<Status>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Fan {
    pub name: Option<String>,
    pub reading: Option<f64>,
    pub reading_units: Option<String>,
    pub status: Option<Status>,
}

/// Processor resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Processor {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub total_cores: Option<u32>,
    pub total_threads: Option<u32>,
    pub max_speed_m_hz: Option<u32>,
    pub status: Option<Status>,
}

/// Memory resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Memory {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub memory_device_type: Option<String>,
    pub capacity_mi_b: Option<u64>,
    pub operating_speed_mhz: Option<u32>,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub status: Option<Status>,
}

/// Storage resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Storage {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub drives: Option<Vec<OdataLink>>,
    pub storage_controllers: Option<Vec<StorageController>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageController {
    pub member_id: Option<String>,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub firmware_version: Option<String>,
    pub status: Option<Status>,
}

/// Drive resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Drive {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub capacity_bytes: Option<u64>,
    pub media_type: Option<String>,
    pub protocol: Option<String>,
    pub status: Option<Status>,
}

/// EthernetInterface resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EthernetInterface {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "MACAddress")]
    pub mac_address: Option<String>,
    pub speed_mbps: Option<u32>,
    pub status: Option<Status>,
    #[serde(rename = "IPv4Addresses")]
    pub ipv4_addresses: Option<Vec<Value>>,
    #[serde(rename = "IPv6Addresses")]
    pub ipv6_addresses: Option<Vec<Value>>,
}

/// Account Service resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountService {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub accounts: Option<OdataLink>,
    pub roles: Option<OdataLink>,
}

/// User Account resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Account {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub user_name: Option<String>,
    pub role_id: Option<String>,
    pub enabled: Option<bool>,
    pub locked: Option<bool>,
}

/// Update Service resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateService {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub firmware_inventory: Option<OdataLink>,
    pub software_inventory: Option<OdataLink>,
    pub service_enabled: Option<bool>,
}

/// Event Service resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EventService {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub service_enabled: Option<bool>,
    pub subscriptions: Option<OdataLink>,
}

/// Log Entry resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LogEntry {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub created: Option<String>,
    pub entry_type: Option<String>,
    pub severity: Option<String>,
    pub message: Option<String>,
    pub message_id: Option<String>,
}

/// Task resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Task {
    #[serde(rename = "@odata.id")]
    pub odata_id: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub task_state: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub task_status: Option<String>,
}

/// Session creation request body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionCreate {
    pub user_name: String,
    pub password: String,
}

/// Reset request body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResetRequest {
    pub reset_type: String,
}

/// Boot override request body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BootOverride {
    pub boot: BootOverrideInner,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BootOverrideInner {
    pub boot_source_override_target: String,
    pub boot_source_override_enabled: Option<String>,
}
