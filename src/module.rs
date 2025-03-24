use crate::event_bus::{Event, EventBus};

use anyhow::Result;
use tokio::sync::broadcast;

pub trait Module {
    fn new(ctx: ModuleCtx) -> Self;
    fn run(&mut self) -> impl Future<Output = Result<()>>;
}

#[derive(Debug)]
pub struct ModuleCtx {
    pub name: String,
    pub sender: broadcast::Sender<Event>,
    pub receiver: broadcast::Receiver<Event>,
}

impl ModuleCtx {
    pub fn new(name: &str, bus: &EventBus) -> Self {
        let sender = bus.sender.clone();
        let receiver = bus.subscribe();

        ModuleCtx {
            name: name.to_string(),
            sender,
            receiver,
        }
    }
}
