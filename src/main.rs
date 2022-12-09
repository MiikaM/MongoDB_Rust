use bson::Timestamp;
use chrono::{DateTime, Utc};
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};
use serde::Deserialize;
use serde_xml_rs::{from_str, to_string};
use std::env;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Report {
    deviceInformation: DeviceInformation,
    capture: Capture,
}

#[derive(Debug, Deserialize)]
struct Capture {
    drone: Vec<Drone>,
}

#[derive(Debug, Deserialize)]
struct Drone {
    serialNumber: String,
    model: String,
    manufacturer: String,
    mac: String,
    ipv4: String,
    ipv6: String,
    firmware: String,
    positionY: f64,
    positionX: f64,
    altitude: f64,
}

#[derive(Debug, Deserialize)]
struct DeviceInformation {
    listenRange: f32,
    deviceStarted: String,
    uptimeSeconds: i64,
    updateIntervalMs: i64,
}

#[tokio::main]
async fn main() {
    parse_xml().await;
}

async fn connect_db() -> Result<(), Box<dyn Error>> {
    let connection_string = env::var("MONGODB_CONNECTION_STRING")
        .expect("$MONGODB_CONNECTION_STRING has not been set!");
    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
        ClientOptions::parse_with_resolver_config(&connection_string, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;

    // Print the databases in our MongoDB cluster:
    println!("Databases:");
    for name in client.list_database_names(None, None).await? {
        println!("- {}", name);
    }
    Ok(())
}

async fn parse_xml() -> Result<(), Box<dyn Error>> {
    let response = reqwest::get("http://assignments.reaktor.com/birdnest/drones")
        .await?
        .text()
        .await?;
    let res: Report = from_str(&response).unwrap();
    
    for drone in res.capture.drone {
        println!("{drone:?}");
    }

    Ok(())
}
