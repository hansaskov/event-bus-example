use crate::module::{Module, ModuleCtx};
use anyhow::Result;

pub struct Network {
    ctx: ModuleCtx,
}

impl Module for Network {
    fn new(ctx: ModuleCtx) -> Self {
        Self { ctx }
    }

    async fn run(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = interval.tick() => self.ctx.send_message("5 seconds has passed".to_string())
            }
        }
    }
}
