# rufish - LLM Integration Guide

This document helps LLMs (AI assistants) understand how to use the `rufish` crate to interact with BMCs via the Redfish REST API.

## Installation

```toml
[dependencies]
rufish = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Core Concepts

- `rufish` is an async Redfish client over HTTPS (port 443)
- All operations require a `tokio` async runtime
- Session lifecycle: `new()` → `login()` → API calls → `logout()`
- Supports session auth (X-Auth-Token) with fallback to Basic Auth
- Accepts self-signed certificates (common for BMCs)
- All resources are typed Rust structs with `Option<>` fields

## Public Types

```rust
use rufish::{
    RedfishClient, RedfishError, Result,
    // Resources
    ServiceRoot, Collection, OdataLink,
    ComputerSystem, Chassis, Manager,
    Power, PowerControl, PowerSupply,
    Thermal, Temperature, Fan,
    Processor, Memory, Storage, Drive,
    EthernetInterface, AccountService, Account,
    UpdateService, EventService, LogEntry, Task,
    // Request types
    Boot, Status,
};
```

## Basic Usage Pattern

```rust
use rufish::RedfishClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RedfishClient::new("10.0.0.5", "admin", "password")?;
    client.login().await?;

    // Use high-level API
    let sys = client.get_system("1").await?;
    println!("Power: {:?}", sys.power_state);

    client.logout().await?;
    Ok(())
}
```

## API Reference

### Constructor

```rust
RedfishClient::new(
    host: &str,       // BMC IP, hostname, or full URL (e.g. "https://bmc.local")
    username: &str,
    password: &str,
) -> Result<Self>
```

### Session Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `login` | `async fn login(&mut self) -> Result<()>` | Establish session (X-Auth-Token) |
| `logout` | `async fn logout(&mut self) -> Result<()>` | Delete session |

### Resource Queries (GET)

| Method | Returns | Endpoint |
|--------|---------|----------|
| `get_service_root()` | `ServiceRoot` | `/redfish/v1/` |
| `list_systems()` | `Collection` | `/redfish/v1/Systems` |
| `get_system(id)` | `ComputerSystem` | `/redfish/v1/Systems/{id}` |
| `list_chassis()` | `Collection` | `/redfish/v1/Chassis` |
| `get_chassis(id)` | `Chassis` | `/redfish/v1/Chassis/{id}` |
| `list_managers()` | `Collection` | `/redfish/v1/Managers` |
| `get_manager(id)` | `Manager` | `/redfish/v1/Managers/{id}` |
| `get_power(chassis_id)` | `Power` | `/redfish/v1/Chassis/{id}/Power` |
| `get_thermal(chassis_id)` | `Thermal` | `/redfish/v1/Chassis/{id}/Thermal` |
| `list_processors(sys_id)` | `Collection` | `/redfish/v1/Systems/{id}/Processors` |
| `get_processor(sys_id, proc_id)` | `Processor` | `.../Processors/{proc_id}` |
| `list_memory(sys_id)` | `Collection` | `/redfish/v1/Systems/{id}/Memory` |
| `get_memory(sys_id, mem_id)` | `Memory` | `.../Memory/{mem_id}` |
| `list_storage(sys_id)` | `Collection` | `/redfish/v1/Systems/{id}/Storage` |
| `get_storage(sys_id, storage_id)` | `Storage` | `.../Storage/{storage_id}` |
| `list_ethernet_interfaces(sys_id)` | `Collection` | `.../EthernetInterfaces` |
| `get_ethernet_interface(sys_id, id)` | `EthernetInterface` | `.../EthernetInterfaces/{id}` |
| `get_account_service()` | `AccountService` | `/redfish/v1/AccountService` |
| `list_accounts()` | `Collection` | `.../AccountService/Accounts` |
| `get_update_service()` | `UpdateService` | `/redfish/v1/UpdateService` |
| `get_event_service()` | `EventService` | `/redfish/v1/EventService` |
| `list_log_entries(mgr_id, log_id)` | `Collection` | `.../LogServices/{log_id}/Entries` |

### Actions (POST/PATCH)

| Method | Signature | Description |
|--------|-----------|-------------|
| `reset_system(id, type)` | `async fn -> Result<Value>` | Any ResetType |
| `power_on(id)` | `async fn -> Result<Value>` | ResetType="On" |
| `power_off(id)` | `async fn -> Result<Value>` | ResetType="ForceOff" |
| `graceful_shutdown(id)` | `async fn -> Result<Value>` | ResetType="GracefulShutdown" |
| `graceful_restart(id)` | `async fn -> Result<Value>` | ResetType="GracefulRestart" |
| `force_restart(id)` | `async fn -> Result<Value>` | ResetType="ForceRestart" |
| `power_cycle(id)` | `async fn -> Result<Value>` | ResetType="PowerCycle" |
| `set_boot_override(id, target, enabled)` | `async fn -> Result<Value>` | Set boot source |
| `set_boot_pxe(id)` | `async fn -> Result<Value>` | Boot to PXE (once) |
| `set_boot_bios(id)` | `async fn -> Result<Value>` | Boot to BIOS Setup (once) |
| `reset_manager(id, type)` | `async fn -> Result<Value>` | Reset BMC |
| `clear_log(mgr_id, log_id)` | `async fn -> Result<Value>` | Clear log service |

### Raw Access

| Method | Signature | Description |
|--------|-----------|-------------|
| `get(path)` | `async fn -> Result<Value>` | GET any endpoint |
| `get_as<T>(path)` | `async fn -> Result<T>` | GET and deserialize |
| `post(path, body)` | `async fn -> Result<Value>` | POST action |
| `patch(path, body)` | `async fn -> Result<Value>` | PATCH resource |
| `delete(path)` | `async fn -> Result<()>` | DELETE resource |

## Examples

### Power Management

```rust
// Check power state
let sys = client.get_system("1").await?;
if sys.power_state.as_deref() == Some("Off") {
    client.power_on("1").await?;
}

// Graceful shutdown
client.graceful_shutdown("1").await?;
```

### Thermal Monitoring

```rust
let thermal = client.get_thermal("1").await?;
for t in thermal.temperatures.unwrap_or_default() {
    println!("{}: {}°C", t.name.unwrap_or_default(), t.reading_celsius.unwrap_or(0.0));
}
for f in thermal.fans.unwrap_or_default() {
    println!("{}: {} {:?}", f.name.unwrap_or_default(), f.reading.unwrap_or(0.0), f.reading_units);
}
```

### Power Consumption

```rust
let power = client.get_power("1").await?;
for pc in power.power_control.unwrap_or_default() {
    println!("Consumed: {}W", pc.power_consumed_watts.unwrap_or(0.0));
}
```

### Inventory

```rust
// CPUs
let procs = client.list_processors("1").await?;
for m in procs.members.unwrap_or_default() {
    let p: rufish::Processor = client.get_as(&m.odata_id).await?;
    println!("{}: {} cores", p.model.unwrap_or_default(), p.total_cores.unwrap_or(0));
}

// Memory
let mems = client.list_memory("1").await?;
for m in mems.members.unwrap_or_default() {
    let mem: rufish::Memory = client.get_as(&m.odata_id).await?;
    println!("{}: {} MiB", mem.name.unwrap_or_default(), mem.capacity_mi_b.unwrap_or(0));
}
```

### Boot Override

```rust
// PXE boot next restart
client.set_boot_pxe("1").await?;

// Custom: boot to USB continuously
client.set_boot_override("1", "Usb", Some("Continuous")).await?;
```

### Raw Endpoint Access

```rust
use serde_json::json;

// GET any path
let val = client.get("/redfish/v1/Systems/1/Bios").await?;

// PATCH to change settings
client.patch("/redfish/v1/Systems/1/Bios/Settings", &json!({
    "Attributes": {
        "BootMode": "Uefi"
    }
})).await?;
```

## Error Handling

```rust
use rufish::RedfishError;

match client.get_system("1").await {
    Ok(sys) => println!("Model: {:?}", sys.model),
    Err(RedfishError::AuthFailed) => println!("Login failed"),
    Err(RedfishError::SessionExpired) => {
        client.login().await?; // Re-authenticate
    }
    Err(RedfishError::NotFound(uri)) => println!("Resource not found: {}", uri),
    Err(RedfishError::Api { status, message }) => println!("HTTP {}: {}", status, message),
    Err(RedfishError::Http(e)) => println!("Network error: {}", e),
    Err(e) => println!("Error: {}", e),
}
```

## Common System IDs

Most single-server BMCs use system ID `"1"` or `"System.Embedded.1"`. Use `list_systems()` to discover available IDs:

```rust
let systems = client.list_systems().await?;
for m in systems.members.unwrap_or_default() {
    println!("System URI: {}", m.odata_id);
    // Extract ID from last path segment
}
```

## ResetType Values

| Value | Description |
|-------|-------------|
| `"On"` | Power on |
| `"ForceOff"` | Immediate power off |
| `"GracefulShutdown"` | OS-initiated shutdown |
| `"GracefulRestart"` | OS-initiated restart |
| `"ForceRestart"` | Immediate restart |
| `"Nmi"` | Non-maskable interrupt |
| `"ForceOn"` | Force power on |
| `"PushPowerButton"` | Simulate button press |
| `"PowerCycle"` | Off then on |

## Boot Source Override Targets

| Value | Description |
|-------|-------------|
| `"None"` | No override |
| `"Pxe"` | PXE network boot |
| `"Cd"` | CD/DVD |
| `"Usb"` | USB device |
| `"Hdd"` | Hard disk |
| `"BiosSetup"` | BIOS/UEFI setup |
| `"Diags"` | Diagnostics |

## Tips for LLMs

1. Always `login()` before making API calls
2. Always `logout()` when done to free BMC session slots
3. Most fields are `Option<T>` — use `.unwrap_or_default()` or pattern matching
4. System/Chassis/Manager IDs vary by vendor — use `list_*()` to discover
5. For bulk operations on many BMCs, create separate `RedfishClient` instances
6. The `get_as::<T>(path)` method works with any path + any deserializable type
7. Use `get(path)` for untyped access when the schema is unknown
8. Session tokens expire — catch `SessionExpired` and re-login
