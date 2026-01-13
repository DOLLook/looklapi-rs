use inventory;
use std::any::Any;

/// 应用事件观察者 trait
/// 实现此 trait 的结构体可以订阅应用运行时事件
pub trait AppObserver {
    /// 处理接收到的应用事件
    /// 注意：实现者需要自行处理可能的 panic
    fn on_application_event(&self, event: &dyn Any);
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

use crate::app::appcontext::publisher::{AppEventPublisher, app_event_publisher};

/// 观察者注册项，用于在静态上下文中存储订阅闭包
pub struct ObserverRegistration {
    pub subscribe_fn: fn(&AppEventPublisher),
}

// 为ObserverRegistration实现inventory的Collect trait
inventory::collect!(ObserverRegistration);

/// 注册所有应用事件观察者
pub fn register_app_observers() {
    let publisher = app_event_publisher();
    // 订阅所有收集到的观察者
    for registration in inventory::iter::<ObserverRegistration>() {
        (registration.subscribe_fn)(&publisher);
    }
}

/// 为实例注册观察者的辅助宏
#[macro_export]
macro_rules! register_observer {
    ($observer:expr, $event_type:ty) => {
        inventory::submit!($crate::app::appcontext::observer::ObserverRegistration {
            subscribe_fn: |publisher| {
                publisher.subscribe::<$event_type>($observer);
            }
        });
    };
}

/// 为类型注册观察者的辅助宏
/// 注意：$type 必须实现 AppObserver trait且必须实现 get_instance() 方法
#[macro_export]
macro_rules! register_observer_for {
    ($type:ty, $event_type:ty) => {
        inventory::submit!($crate::app::appcontext::observer::ObserverRegistration {
            subscribe_fn: |publisher| {
                publisher.subscribe::<$event_type>(<$type>::get_instance());
            }
        });
    };
}

/// 为类型注册多个事件类型的观察者
#[macro_export]
macro_rules! register_observer_for_events {
    ($type:ty, $($event_type:ty),+) => {
        $(
            $crate::register_observer_for!($type, $event_type);
        )+
    };
}
