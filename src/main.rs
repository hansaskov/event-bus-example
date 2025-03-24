pub mod event_bus;
pub mod module;
pub mod modules;

use anyhow::Result;
use event_bus::EventBus;
use module::{Module, ModuleCtx};
use modules::logger::Logger;
use modules::monitoring::Monitoring;
use modules::network::Network;

#[tokio::main]
async fn main() -> Result<()> {
    let event_bus = EventBus::new();

    let logger_ctx = ModuleCtx::new("logger", &event_bus);
    let mut logger = Logger::new(logger_ctx);

    let network_ctx = ModuleCtx::new("network", &event_bus);
    let mut network = Network::new(network_ctx);

    let monitoring_ctx = ModuleCtx::new("monitoring", &event_bus);
    let mut monitoring = Monitoring::new(monitoring_ctx);

    tokio::join!(network.run(), logger.run(), monitoring.run()).0?;

    Ok(())
}
