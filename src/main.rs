use actix::{Actor, Arbiter, System};
use log::{error, LevelFilter};

use home_automation_tapo::system::coordinator_actor::CoordinatorActor;

fn main() {
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);

    pretty_env_logger::formatted_timed_builder()
        .filter(Some("home_automation_tapo"), log_level)
        .init();

    let system = System::new();

    let execution = async {
        let coordinator = CoordinatorActor::new();

        coordinator.start();
    };

    let arbiter = Arbiter::new();
    arbiter.spawn(execution);

    let result = system.run();

    if let Err(e) = result {
        error!("System could not start {}", e);
    }
}
