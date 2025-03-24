use std::time::SystemTime;

use crate::event_bus::{Event, EventKind, Reading};
use crate::module::{Module, ModuleCtx};
use anyhow::Result;

pub struct Monitoring {
    ctx: ModuleCtx,
}

impl Module for Monitoring {
    fn new(ctx: ModuleCtx) -> Self {
        Monitoring { ctx }
    }

    async fn run(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let reading = Reading{
                        time: SystemTime::now(),
                        name: "CPU Temp".into(),
                        category: "computer".into(),
                        unit: "C".into(),
                        value: 20.0
                    };

                    let event = Event {
                        module: self.ctx.name.to_string(),
                        inner: EventKind::Reading(reading),
                    };
                    self.ctx.sender
                        .send(event)
                        .unwrap();
                },
            }
        }
    }
}
