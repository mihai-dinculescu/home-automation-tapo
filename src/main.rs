use actix::Actor;
use log::{LevelFilter, info};

use home_automation_tapo::system::coordinator_actor::CoordinatorActor;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    pretty_env_logger::formatted_timed_builder()
        .filter(Some("home_automation_tapo"), log_level)
        .init();

    info!("Starting home automation tapo system with Actix-RT on Tokio runtime");

    // Start coordinator actor
    let coordinator = CoordinatorActor::new();
    coordinator.start();

    info!("System started, waiting for shutdown signal...");

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");

    info!("Received shutdown signal, shutting down application...");

    Ok(())
}
