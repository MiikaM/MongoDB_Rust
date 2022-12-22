use bson::Timestamp;
use chrono::{ Utc, Date, Local};
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
    bson::{DateTime},
    
};
use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;
use std::env;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Report {
    deviceInformation: DeviceInformation,
    capture: Capture,
}

#[derive(Debug, Deserialize)]
struct Capture {
    snapshotTimestamp: String,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pilot {
    pilotId: String,
    firstName: String,
    lastName: String,
    phoneNumber: String,
    createdDt: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct DeviceInformation {
    listenRange: f32,
    deviceStarted: String,
    uptimeSeconds: i64,
    updateIntervalMs: i64,
}

const NEST_POSITION_X: f64 = 250000.0;
const NEST_POSITION_Y: f64 = 250000.0;
const NO_FLY_ZONE_SIZE: f64 = 100.0;

#[tokio::main]
async fn main() {
    let report: Report = parse_drone_information().await;
    let timestamp = DateTime::parse_rfc3339_str(&report.capture.snapshotTimestamp);
    let drone_violations: Vec<String> = find_drone_violations(&report.capture.drone);
    let pilot_information: Vec<Pilot> = get_pilot_information(&drone_violations).await;
    println!("{pilot_information:?}");
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

async fn parse_drone_information() -> Report {
    let response = reqwest::get("http://assignments.reaktor.com/birdnest/drones").await;
    let res_text = response
        .expect("The response couldn't be parsed")
        .text()
        .await
        .unwrap();
    let res: Report = from_str(&res_text).unwrap();
    res
}

fn find_drone_violations(drones: &Vec<Drone>) -> Vec<String> {
    let mut drone_violations: Vec<String> = vec![];
    for drone in drones {
        let dist_drone_x = (drone.positionX - NEST_POSITION_X).abs();
        let dist_drone_y = (drone.positionY - NEST_POSITION_Y).abs();
        let dist_from_nest: f64 =
            ((dist_drone_x * dist_drone_x) + (dist_drone_y * dist_drone_y)).sqrt();

        if dist_from_nest <= 100000.0 {
            drone_violations.push(drone.serialNumber.clone());
        }
    }

    drone_violations
}

async fn get_pilot_information(drone_srlNumbers: &Vec<String>) -> Vec<Pilot> {
    let mut pilots: Vec<Pilot> = vec![];
    for serialNumber in drone_srlNumbers {
        let query_uri = format!("http://assignments.reaktor.com/birdnest/pilots/{serialNumber}");
        let response = reqwest::get(query_uri).await;
        let res_text = response.expect("get").text().await.unwrap();
        let pilot_info: Pilot = serde_json::from_str(&res_text).unwrap();

        println!("res_text: {pilot_info:?}");
        pilots.push(pilot_info);
    }

    pilots
}
