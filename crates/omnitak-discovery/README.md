# omnitak-discovery

Multicast DNS (mDNS) service discovery for TAK infrastructure.

## Overview

This crate provides automatic network discovery capabilities for TAK servers, ATAK devices, and other TAK aggregators using mDNS (RFC 6762) and DNS-SD (RFC 6763).

## Features

- Automatic discovery of TAK servers advertising via mDNS
- ATAK device discovery on local networks
- Service announcement for OmniTAK aggregator
- Health tracking and stale service cleanup
- Event-based notifications for service changes
- RESTful API integration

## Usage

```rust
use omnitak_discovery::{DiscoveryService, DiscoveryConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create discovery service
    let config = DiscoveryConfig::default();
    let discovery = DiscoveryService::new(config)?;

    // Start discovery
    discovery.start().await?;

    // Get discovered services
    let servers = discovery.get_discovered_services().await;
    for service in servers {
        println!("Found: {} at {}", service.instance_name, service.connection_string());
    }

    Ok(())
}
```

## Configuration

Add to your `config.yaml`:

```yaml
discovery:
  enabled: true
  announce_enabled: true
  announce_port: 8080
  service_types:
    - tak_server
    - atak_device
    - tak_aggregator
  auto_connect: false
  require_tls: true
  cleanup_interval_secs: 30
  stale_timeout_secs: 300
```

## REST API Endpoints

When integrated with the API server:

- `GET /api/v1/discovery/status` - Discovery service status
- `GET /api/v1/discovery/services` - List all discovered services
- `GET /api/v1/discovery/services/:id` - Get specific service details
- `POST /api/v1/discovery/refresh` - Manually trigger refresh
- `GET /api/v1/discovery/tak-servers` - List TAK servers only
- `GET /api/v1/discovery/atak-devices` - List ATAK devices only

## Service Types

- **TAK Server** (`_tak._tcp.local.`) - CoT streaming servers
- **ATAK Device** (`_atak._tcp.local.`) - Android ATAK devices
- **TAK Aggregator** (`_tak-aggregator._tcp.local.`) - Other aggregators

## mDNS Implementation

Currently uses the `mdns-sd` crate which provides good RFC compliance. The architecture is designed to allow swapping implementations if enhanced RFC 6762 compliance or custom features are required.

## Security Considerations

- Discovery is limited to local network segments
- TLS verification is recommended for auto-connect scenarios
- Services discovered via mDNS should be validated before use
- Manual approval is recommended for production environments

## License

MIT OR Apache-2.0
