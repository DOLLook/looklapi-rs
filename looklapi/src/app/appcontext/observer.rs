use std::any::Any;

/// 应用事件观察者 trait
/// 实现此 trait 的结构体可以订阅应用运行时事件
pub trait AppObserver {
    /// 处理接收到的应用事件
    /// 注意：实现者需要自行处理可能的 panic
    fn on_application_event(&self, event: &dyn Any);

    /// 注册到应用事件发布器
    fn subscribe(&self);
}

/// 安全地调用观察者的事件处理方法
/// 捕获并记录可能的 panic
pub(crate) fn on_event(observer: &dyn AppObserver, event: &dyn Any) {
    if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        observer.on_application_event(event);
    })) {
        if let Some(msg) = panic.downcast_ref::<&str>() {
            tracing::error!("Observer panic: {}", msg);
        } else if let Some(msg) = panic.downcast_ref::<String>() {
            tracing::error!("Observer panic: {}", msg);
        } else {
            tracing::error!("Observer panic: unknown error");
        }
    }
}