use crate::event_bus::{Event, EventBus, EventKind, Reading};

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

    pub fn send(&self, event_kind: EventKind) {
        let event = Event {
            module: self.name.clone(),
            inner: event_kind,
        };

        self.sender.send(event).unwrap();
    }

    pub fn send_message(&self, message: String) {
        self.send(EventKind::Message(message));
    }

    pub fn send_reading(&self, reading: Reading) {
        self.send(EventKind::Reading(reading));
    }
}
