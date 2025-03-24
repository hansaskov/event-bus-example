use crate::event_bus::{Event, EventKind};
use crate::module::{Module, ModuleCtx};
use anyhow::Result;

pub struct Network {
    ctx: ModuleCtx,
}

impl Module for Network {
    fn new(ctx: ModuleCtx) -> Self {
        Network { ctx }
    }

    async fn run(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let event = Event {
                        module: self.ctx.name.to_string(),
                        inner: EventKind::Message("Completed some work".to_string()),
                    };
                    self.ctx.sender
                        .send(event)
                        .unwrap();
                },
            }
        }
    }
}
