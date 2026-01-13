use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

pub struct RudiContext {
    ctx: rudi::Context,
}

// 手动实现 Send 和 Sync trait，因为 rudi::Context 包含 Box<dyn Any> 字段
unsafe impl Send for RudiContext {}
unsafe impl Sync for RudiContext {}

impl RudiContext {
    fn new(ctx: rudi::Context) -> Self {
        Self { ctx }
    }

    pub fn get_ctx(&self) -> &rudi::Context {
        &self.ctx
    }

    pub fn get_ctx_mut(&mut self) -> &mut rudi::Context {
        &mut self.ctx
    }
}

// 实现单例模式
static RUDI_CONTEXT: OnceLock<Arc<RwLock<RudiContext>>> = OnceLock::new();

impl RudiContext {
    pub fn instance() -> Arc<RwLock<RudiContext>> {
        RUDI_CONTEXT
            .get_or_init(|| {
                let ctx = rudi::Context::options().eager_create(true).auto_register();
                let rudi_context = RudiContext::new(ctx);
                Arc::new(RwLock::new(rudi_context))
            })
            .clone()
    }
}
