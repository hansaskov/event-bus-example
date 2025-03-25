use crate::module::{Module, ModuleCtx};
use anyhow::Result;

pub struct Network {
    ctx: ModuleCtx,
}

impl Network {
    pub fn new(ctx: ModuleCtx) -> Self {
        Self { ctx }
    }
}

impl Module for Network {
    async fn run(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = interval.tick() => self.ctx.send_log("5 seconds has passed".to_string())
            }
        }
    }
}
