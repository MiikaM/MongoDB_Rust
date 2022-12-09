use mongodb::{options::{ClientOptions, ResolverConfig}, Client};
use std::env;
use std::error::Error;

#[tokio::main]
async fn main()  {
    connect_db().await;
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