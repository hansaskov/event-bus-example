use crate::event_bus::EventKind;
use crate::module::{Module, ModuleCtx};
use anyhow::Result;

pub struct Logger {
    ctx: ModuleCtx,
}

impl Module for Logger {
    fn new(ctx: ModuleCtx) -> Self {
        Self { ctx }
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                e = self.ctx.receiver.recv() => {
                    match e {
                        Ok(event) => {
                            match event.inner {
                                EventKind::Message(message) => println!("{}: received event from {}: {}",  &self.ctx.name, event.module, message),
                                EventKind::Reading(reading) => println!("{}: received reading: {} {} {}", &self.ctx.name, reading.name, reading.value, reading.unit)
                            }
                        },
                        Err(e) => println!("Error: {e}")
                    }
                },
            }
        }
    }
}
