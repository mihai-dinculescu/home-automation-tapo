use actix::Actor;
use log::info;

use home_automation_tapo::settings::Settings;
use home_automation_tapo::system::coordinator_actor::CoordinatorActor;
use home_automation_tapo::telemetry::{init_telemetry, shutdown_telemetry};

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::new().expect("failed to read the settings");

    let (logger_provider, tracer_provider, meter_provider) = init_telemetry(&settings.telemetry)?;

    info!("Starting home automation tapo system with Actix-RT on Tokio runtime");

    // Start coordinator actor
    let coordinator = CoordinatorActor::new(settings);
    coordinator.start();

    info!("System started, waiting for shutdown signal...");

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");

    info!("Received shutdown signal, shutting down application...");

    // Shutdown telemetry
    shutdown_telemetry(logger_provider, tracer_provider, meter_provider)?;

    Ok(())
}
