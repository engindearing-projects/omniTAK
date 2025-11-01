//! omnitak-gen - CLI tool for generating CoT messages
//!
//! Generate test Cursor on Target messages in XML and binary formats
//! for testing TAK servers and omniTAK aggregators.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use clap::{Parser, ValueEnum};
use std::io::Write;
use std::net::{TcpStream, UdpSocket};
use uuid::Uuid;

/// Generate CoT messages for testing TAK servers
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Latitude in decimal degrees
    #[arg(long, required = true)]
    lat: f64,

    /// Longitude in decimal degrees
    #[arg(long, required = true)]
    lon: f64,

    /// Altitude in meters (Height Above Ellipsoid)
    #[arg(long, default_value = "0.0")]
    alt: f64,

    /// Callsign/name for the entity
    #[arg(long)]
    callsign: Option<String>,

    /// Unique ID (auto-generated UUID if not specified)
    #[arg(long)]
    uid: Option<String>,

    /// Affiliation type
    #[arg(long, value_enum, default_value = "friendly")]
    affiliation: AffiliationType,

    /// Entity type
    #[arg(long, value_enum, default_value = "ground")]
    entity: EntityType,

    /// Speed in meters per second
    #[arg(long, default_value = "0.0")]
    speed: f64,

    /// Course/heading in degrees (0-360)
    #[arg(long, default_value = "0.0")]
    course: f64,

    /// Stale time in minutes
    #[arg(long, default_value = "5")]
    stale: i64,

    /// Output format
    #[arg(long, value_enum, default_value = "xml")]
    format: OutputFormat,

    /// Output file (stdout if not specified)
    #[arg(long, short)]
    output: Option<String>,

    /// Generate multiple messages
    #[arg(long, default_value = "1")]
    batch: usize,

    /// Interval between batch messages in milliseconds
    #[arg(long, default_value = "1000")]
    interval: u64,

    /// Send to TAK server instead of writing to file
    #[arg(long)]
    send: bool,

    /// TAK server address (HOST:PORT)
    #[arg(long, default_value = "localhost:8087")]
    server: String,

    /// Protocol for sending
    #[arg(long, value_enum, default_value = "udp")]
    protocol: Protocol,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AffiliationType {
    Pending,
    Unknown,
    AssumedFriend,
    Friendly,
    Neutral,
    Suspect,
    Hostile,
}

impl AffiliationType {
    fn to_code(&self) -> char {
        match self {
            AffiliationType::Pending => 'p',
            AffiliationType::Unknown => 'u',
            AffiliationType::AssumedFriend => 'a',
            AffiliationType::Friendly => 'f',
            AffiliationType::Neutral => 'n',
            AffiliationType::Suspect => 's',
            AffiliationType::Hostile => 'h',
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum EntityType {
    Ground,
    Air,
    Sea,
}

impl EntityType {
    fn to_cot_type(&self, affiliation: AffiliationType) -> String {
        let aff_code = affiliation.to_code();
        match self {
            EntityType::Ground => format!("a-{}-G-E-V", aff_code), // Ground Equipment Vehicle
            EntityType::Air => format!("a-{}-A-M-F", aff_code),    // Air Military Fixed Wing
            EntityType::Sea => format!("a-{}-S-S-F", aff_code),    // Sea Surface Ship
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Xml,
    Mesh,   // TAK Protocol Version 1 - Mesh
    Stream, // TAK Protocol Version 1 - Stream
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Protocol {
    Udp,
    Tcp,
    Tls,
}

// Simple event structure for message generation
#[derive(Debug, Clone)]
struct CotEvent {
    version: String,
    uid: String,
    event_type: String,
    time: DateTime<Utc>,
    start: DateTime<Utc>,
    stale: DateTime<Utc>,
    how: String,
    lat: f64,
    lon: f64,
    hae: f64,
    ce: f64,
    le: f64,
    callsign: String,
    speed: f64,
    course: f64,
    remarks: String,
}

struct MessageGenerator {
    args: Args,
}

impl MessageGenerator {
    fn new(args: Args) -> Self {
        Self { args }
    }

    fn generate_event(&self, index: usize) -> CotEvent {
        let now = Utc::now();
        let stale_time = now + Duration::minutes(self.args.stale);

        // Generate UID
        let uid = self.args.uid.clone().unwrap_or_else(|| {
            if self.args.batch > 1 {
                format!("omnitak-gen-{}-{}", Uuid::new_v4(), index)
            } else {
                format!("omnitak-gen-{}", Uuid::new_v4())
            }
        });

        // Generate callsign
        let callsign = self.args.callsign.clone().unwrap_or_else(|| {
            if self.args.batch > 1 {
                format!("UNIT-{}", index + 1)
            } else {
                "UNIT-1".to_string()
            }
        });

        // Calculate position for batch generation (spread along latitude)
        let lat_offset = (index as f64) * 0.001; // ~111 meters per unit
        let lat = self.args.lat + lat_offset;

        // Build CoT type
        let cot_type = self.args.entity.to_cot_type(self.args.affiliation);

        CotEvent {
            version: "2.0".to_string(),
            uid,
            event_type: cot_type,
            time: now,
            start: now,
            stale: stale_time,
            how: "m-g".to_string(), // machine generated
            lat,
            lon: self.args.lon,
            hae: self.args.alt,
            ce: 9999999.0,
            le: 9999999.0,
            callsign,
            speed: self.args.speed,
            course: self.args.course,
            remarks: format!("Generated by omnitak-gen - {:?}", self.args.affiliation),
        }
    }

    fn event_to_xml(&self, event: &CotEvent) -> Result<String> {
        let detail_xml = format!(
            r#"<detail>
    <contact callsign="{}"/>
    <track speed="{}" course="{}"/>
    <remarks>{}</remarks>
  </detail>"#,
            event.callsign, event.speed, event.course, event.remarks
        );

        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="{}" uid="{}" type="{}" how="{}" time="{}" start="{}" stale="{}">
  <point lat="{}" lon="{}" hae="{}" ce="{}" le="{}"/>
  {}
</event>"#,
            event.version,
            event.uid,
            event.event_type,
            event.how,
            event.time.to_rfc3339(),
            event.start.to_rfc3339(),
            event.stale.to_rfc3339(),
            event.lat,
            event.lon,
            event.hae,
            event.ce,
            event.le,
            detail_xml
        );

        Ok(xml)
    }

    fn event_to_binary(&self, event: &CotEvent, format: OutputFormat) -> Result<Vec<u8>> {
        // For now, we'll create a simplified binary format
        // In a full implementation, this would use proper protobuf encoding

        match format {
            OutputFormat::Mesh | OutputFormat::Stream => {
                // For testing purposes, wrap the XML in TAK binary protocol headers
                let xml = self.event_to_xml(event)?;
                let xml_bytes = xml.as_bytes();

                let magic = match format {
                    OutputFormat::Mesh => 0xbf01bf01u32,
                    OutputFormat::Stream => 0xbfbfbfbfu32,
                    _ => unreachable!(),
                };

                let mut buffer = Vec::new();
                buffer.extend_from_slice(&magic.to_le_bytes());
                buffer.extend_from_slice(&(xml_bytes.len() as u32).to_le_bytes());
                buffer.extend_from_slice(xml_bytes);
                Ok(buffer)
            }
            OutputFormat::Xml => unreachable!(),
        }
    }

    fn send_udp(&self, data: &[u8]) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").context("Failed to bind UDP socket")?;
        socket
            .send_to(data, &self.args.server)
            .context("Failed to send UDP packet")?;

        if self.args.verbose {
            eprintln!("Sent {} bytes via UDP to {}", data.len(), self.args.server);
        }

        Ok(())
    }

    fn send_tcp(&self, data: &[u8]) -> Result<()> {
        let mut stream =
            TcpStream::connect(&self.args.server).context("Failed to connect via TCP")?;
        stream.write_all(data).context("Failed to send TCP data")?;

        if self.args.verbose {
            eprintln!("Sent {} bytes via TCP to {}", data.len(), self.args.server);
        }

        Ok(())
    }

    fn send_message(&self, data: &[u8]) -> Result<()> {
        match self.args.protocol {
            Protocol::Udp => self.send_udp(data),
            Protocol::Tcp => self.send_tcp(data),
            Protocol::Tls => {
                anyhow::bail!("TLS protocol not yet implemented. Use UDP or TCP for now.");
            }
        }
    }

    fn write_output(&self, data: &[u8]) -> Result<()> {
        if let Some(output_file) = &self.args.output {
            std::fs::write(output_file, data)
                .with_context(|| format!("Failed to write to file: {}", output_file))?;
            if self.args.verbose {
                eprintln!("Wrote {} bytes to {}", data.len(), output_file);
            }
        } else {
            std::io::stdout()
                .write_all(data)
                .context("Failed to write to stdout")?;
        }
        Ok(())
    }

    fn run(&self) -> Result<()> {
        for i in 0..self.args.batch {
            let event = self.generate_event(i);

            if self.args.verbose {
                eprintln!("Generated event {}/{}", i + 1, self.args.batch);
                eprintln!("  UID: {}", event.uid);
                eprintln!("  Type: {}", event.event_type);
                eprintln!("  Position: {}, {}, {}", event.lat, event.lon, event.hae);
            }

            let data = match self.args.format {
                OutputFormat::Xml => self.event_to_xml(&event)?.into_bytes(),
                OutputFormat::Mesh | OutputFormat::Stream => {
                    self.event_to_binary(&event, self.args.format)?
                }
            };

            if self.args.send {
                self.send_message(&data)?;
            } else {
                self.write_output(&data)?;
            }

            // Sleep between messages in batch mode
            if i + 1 < self.args.batch {
                std::thread::sleep(std::time::Duration::from_millis(self.args.interval));
            }
        }

        if self.args.send {
            println!(
                "Successfully sent {} message(s) to {}",
                self.args.batch, self.args.server
            );
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate coordinates
    if args.lat < -90.0 || args.lat > 90.0 {
        anyhow::bail!("Latitude must be between -90 and 90");
    }
    if args.lon < -180.0 || args.lon > 180.0 {
        anyhow::bail!("Longitude must be between -180 and 180");
    }
    if args.course < 0.0 || args.course > 360.0 {
        anyhow::bail!("Course must be between 0 and 360");
    }

    let generator = MessageGenerator::new(args);
    generator.run()
}
