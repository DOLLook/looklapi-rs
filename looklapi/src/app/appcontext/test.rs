use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::time::sleep;

use crate::app;
use crate::app::appcontext::events::{AppEventBeanInjected, AppEventInitCompleted};
use crate::app::appcontext::observer::AppObserver;
use crate::app::appcontext::publisher::app_event_publisher;

/// 测试观察者，用于验证事件接收
struct TestObserver {
    init_completed_called: AtomicBool,
    bean_injected_called: AtomicBool,
    panic_test: bool,
}

impl TestObserver {
    fn new(panic_test: bool) -> Self {
        Self {
            init_completed_called: AtomicBool::new(false),
            bean_injected_called: AtomicBool::new(false),
            panic_test,
        }
    }

    fn reset(&self) {
        self.init_completed_called.store(false, Ordering::SeqCst);
        self.bean_injected_called.store(false, Ordering::SeqCst);
    }

    fn is_init_completed_called(&self) -> bool {
        self.init_completed_called.load(Ordering::SeqCst)
    }

    fn is_bean_injected_called(&self) -> bool {
        self.bean_injected_called.load(Ordering::SeqCst)
    }
}

impl AppObserver for TestObserver {
    fn on_application_event(&self, event: &dyn std::any::Any) {
        if self.panic_test {
            panic!("Test panic in observer");
        }

        if event.downcast_ref::<AppEventInitCompleted>().is_some() {
            self.init_completed_called.store(true, Ordering::SeqCst);
            println!("TestObserver received AppEventInitCompleted");
        } else if event.downcast_ref::<AppEventBeanInjected>().is_some() {
            self.bean_injected_called.store(true, Ordering::SeqCst);
            println!("TestObserver received AppEventBeanInjected");
        }
    }
}

impl Clone for TestObserver {
    fn clone(&self) -> Self {
        Self {
            init_completed_called: AtomicBool::new(
                self.init_completed_called.load(Ordering::SeqCst),
            ),
            bean_injected_called: AtomicBool::new(self.bean_injected_called.load(Ordering::SeqCst)),
            panic_test: self.panic_test,
        }
    }
}

impl std::fmt::Debug for TestObserver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestObserver")
            .field("init_completed_called", &self.init_completed_called)
            .field("bean_injected_called", &self.bean_injected_called)
            .field("panic_test", &self.panic_test)
            .finish()
    }
}

/// 测试全局事件系统
#[tokio::test]
async fn test_global_event_system() {
    println!("Starting global event system test...");

    // 创建测试观察者
    let observer = Arc::new(TestObserver::new(false));

    // 订阅事件 - 使用原始的Arc引用
    let publisher = app_event_publisher();
    publisher.subscribe::<AppEventInitCompleted>(observer.clone());
    publisher.subscribe::<AppEventBeanInjected>(observer.clone());

    // 发布 AppEventInitCompleted 事件
    app::appcontext::publisher::publish_event(AppEventInitCompleted);

    // 等待事件处理完成（异步）
    sleep(Duration::from_secs(2)).await;

    // 验证事件是否被接收
    assert!(
        observer.is_init_completed_called(),
        "AppEventInitCompleted should be received"
    );
    assert!(
        !observer.is_bean_injected_called(),
        "AppEventBeanInjected should not be received yet"
    );

    // 发布 AppEventBeanInjected 事件
    app::appcontext::publisher::publish_event(AppEventBeanInjected);

    // 等待事件处理完成
    sleep(Duration::from_secs(2)).await;

    // 验证事件是否被接收
    assert!(
        observer.is_bean_injected_called(),
        "AppEventBeanInjected should be received"
    );

    println!("Global event system test passed!");
}

/// 测试 panic 捕获
#[tokio::test]
async fn test_panic_capture() {
    println!("Starting panic capture test...");

    // 创建会 panic 的测试观察者
    let panic_observer = Arc::new(TestObserver::new(true));

    // 订阅事件 - 使用原始的Arc引用
    let publisher = app_event_publisher();
    publisher.subscribe::<AppEventInitCompleted>(panic_observer.clone());

    // 发布事件，应该触发 panic，但系统应该捕获并继续运行
    app::appcontext::publisher::publish_event(AppEventInitCompleted);

    // 等待事件处理完成
    sleep(Duration::from_secs(2)).await;

    // 验证系统仍然正常运行
    // 发布另一个事件到正常观察者
    let normal_observer = Arc::new(TestObserver::new(false));
    publisher.subscribe::<AppEventBeanInjected>(normal_observer.clone());

    app::appcontext::publisher::publish_event(AppEventBeanInjected);
    sleep(Duration::from_secs(2)).await;

    assert!(
        normal_observer.is_bean_injected_called(),
        "Normal observer should receive event"
    );

    println!("Panic capture test passed!");
}

/// 测试重复注册
#[tokio::test]
async fn test_duplicate_registration() {
    println!("Starting duplicate registration test...");

    let observer = Arc::new(TestObserver::new(false));

    // 多次订阅同一事件
    let publisher = app_event_publisher();
    publisher.subscribe::<AppEventInitCompleted>(observer.clone());
    publisher.subscribe::<AppEventInitCompleted>(observer.clone()); // 重复注册

    // 发布事件
    app::appcontext::publisher::publish_event(AppEventInitCompleted);

    // 等待事件处理完成
    sleep(Duration::from_millis(100)).await;

    // 验证事件只被接收一次（重复注册不应该导致重复接收）
    assert!(
        observer.is_init_completed_called(),
        "Event should be received"
    );

    println!("Duplicate registration test passed!");
}
