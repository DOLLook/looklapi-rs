pub const LOG_LEVEL_CHANGE: &str = "log_level_change"; // 日志等级变更交换器
pub const MANUAL_SERVICE_REFRESH: &str = "manual_service_refresh"; // 服务配置刷新交换器
pub const CONFIG_REFRESH_WATCH: &str = "config_refresh_watch"; // 配置刷新交换器

// 消费者类型枚举
#[derive(Debug, Clone, Copy)]
pub enum ConsumerType {
    Invalid,
    WorkQueue,
    Broadcast,
    Topic,
}

impl Default for ConsumerType {
    fn default() -> Self {
        ConsumerType::Invalid
    }
}
