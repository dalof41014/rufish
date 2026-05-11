# rufish

[![Crates.io](https://img.shields.io/crates/v/rufish.svg)](https://crates.io/crates/rufish)
[![Docs.rs](https://docs.rs/rufish/badge.svg)](https://docs.rs/rufish)
[![License](https://img.shields.io/crates/l/rufish.svg)](https://github.com/dalof41014/rufish/blob/main/LICENSE-MIT)

**`rufish`** is an asynchronous **Redfish client library** written in Rust for BMC/server out-of-band management via the DMTF Redfish REST API.

---

## Features

* ✅ Async HTTP/HTTPS client (based on `reqwest` + `tokio`)
* ✅ Session-based authentication (X-Auth-Token) with fallback to Basic Auth
* ✅ Self-signed certificate support (common for BMCs)
* ✅ Typed Rust structs for all major Redfish resources
* ✅ High-level API for common operations:
  * Systems, Chassis, Managers
  * Power & Thermal monitoring
  * Processors, Memory, Storage, Drives
  * Ethernet Interfaces
  * Power control (On/Off/Restart/Cycle)
  * Boot override (PXE, BIOS Setup, HDD, etc.)
  * Account management
  * Log services
  * Update & Event services
* ✅ Low-level GET/POST/PATCH/DELETE for any Redfish endpoint
* ✅ Proper error handling with typed errors

---

## Quick Start

```rust
use rufish::RedfishClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RedfishClient::new("10.0.0.5", "admin", "password")?;
    client.login().await?;

    // Service Root
    let root = client.get_service_root().await?;
    println!("Redfish version: {:?}", root.redfish_version);

    // List systems
    let systems = client.list_systems().await?;
    println!("Systems: {:?}", systems.members_count);

    // Get system details
    let sys = client.get_system("1").await?;
    println!("Model: {:?}, Power: {:?}", sys.model, sys.power_state);

    // Power control
    client.power_on("1").await?;
    client.graceful_shutdown("1").await?;

    // Thermal monitoring
    let thermal = client.get_thermal("1").await?;
    for t in thermal.temperatures.unwrap_or_default() {
        println!("{}: {}°C", t.name.unwrap_or_default(), t.reading_celsius.unwrap_or(0.0));
    }

    // Boot override
    client.set_boot_pxe("1").await?;

    client.logout().await?;
    Ok(())
}
```

---

## API Summary

### Session

| Method | Description |
|--------|-------------|
| `new(host, user, pass)` | Create client (HTTPS, accepts self-signed certs) |
| `login()` | Establish Redfish session |
| `logout()` | Close session |

### Resources (GET)

| Method | Description |
|--------|-------------|
| `get_service_root()` | Service Root |
| `list_systems()` / `get_system(id)` | Computer Systems |
| `list_chassis()` / `get_chassis(id)` | Chassis |
| `list_managers()` / `get_manager(id)` | Managers (BMC) |
| `get_power(chassis_id)` | Power readings & supplies |
| `get_thermal(chassis_id)` | Temperatures & fans |
| `list_processors(sys)` / `get_processor(sys, id)` | CPUs |
| `list_memory(sys)` / `get_memory(sys, id)` | DIMMs |
| `list_storage(sys)` / `get_storage(sys, id)` | Storage controllers |
| `list_ethernet_interfaces(sys)` | NICs |
| `get_account_service()` / `list_accounts()` | User accounts |
| `get_update_service()` | Firmware update service |
| `get_event_service()` | Event subscriptions |
| `list_log_entries(mgr, log)` | Log entries |

### Actions (POST/PATCH)

| Method | Description |
|--------|-------------|
| `reset_system(id, type)` | Reset with any ResetType |
| `power_on(id)` | Power on |
| `power_off(id)` | Force power off |
| `graceful_shutdown(id)` | ACPI shutdown |
| `graceful_restart(id)` | Graceful restart |
| `force_restart(id)` | Force restart |
| `power_cycle(id)` | Power cycle |
| `set_boot_override(id, target, enabled)` | Set boot source |
| `set_boot_pxe(id)` | Boot to PXE |
| `set_boot_bios(id)` | Boot to BIOS Setup |
| `reset_manager(id, type)` | Reset BMC |
| `clear_log(mgr, log)` | Clear log service |

### Raw Access

| Method | Description |
|--------|-------------|
| `get(path)` | GET any endpoint (returns `serde_json::Value`) |
| `post(path, body)` | POST action |
| `patch(path, body)` | PATCH resource |
| `delete(path)` | DELETE resource |

---

## Dependencies

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
log = "0.4"
```

---

## License

MIT OR Apache-2.0
