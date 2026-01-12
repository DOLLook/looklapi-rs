use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::runtime::Handle;

use crate::app::appcontext::observer::{AppObserver, on_event};

/// 应用事件发布器
/// 在应用运行时是单例实例
pub struct AppEventPublisher {
    /// 事件类型到观察者列表的映射
    observers: Mutex<std::collections::HashMap<TypeId, Vec<Arc<dyn AppObserver + Send + Sync>>>>,
    /// Tokio运行时句柄，用于异步处理事件
    rt_handle: Handle,
}

/// 事件发布器单例
static EVENT_PUBLISHER: OnceLock<Arc<AppEventPublisher>> = OnceLock::new();

/// 获取应用事件发布器单例
pub fn get_app_event_publisher() -> Arc<AppEventPublisher> {
    EVENT_PUBLISHER.get_or_init(|| {
        Arc::new(AppEventPublisher {
            observers: Mutex::new(std::collections::HashMap::new()),
            rt_handle: Handle::current(),
        })
    }).clone()
}

impl AppEventPublisher {
    /// 注册观察者到应用事件发布器
    /// 
    /// # 参数
    /// * `observer` - 要注册的观察者
    /// * `E` - 观察者感兴趣的事件类型
    pub fn subscribe<E: Any + 'static>(&self, observer: Arc<dyn AppObserver + Send + Sync>) {
        let event_type_id = TypeId::of::<E>();
        let mut observers = self.observers.lock().unwrap();
        
        // 检查观察者是否已经注册
        let mut need_add = true;
        if let Some(obs) = observers.get(&event_type_id) {
            for ob in obs {
                if Arc::ptr_eq(ob, &observer) {
                    need_add = false;
                    break;
                }
            }
        }
        
        if need_add {
            observers.entry(event_type_id)
                .or_insert_with(Vec::new)
                .push(observer);
            println!("Observer subscribed for event type: {:?}", event_type_id);
        }
    }

    /// 发布事件到已注册的观察者
    /// 
    /// # 参数
    /// * `event` - 要发布的事件
    pub fn publish_event<E: Any + Send + Sync + 'static>(&self, event: E) {
        let event_type_id = TypeId::of::<E>();
        println!("Publishing event with type ID: {:?}", event_type_id);
        
        let observers = self.observers.lock().unwrap();
        
        if let Some(obs) = observers.get(&event_type_id) {
            println!("Found {} observers for event type", obs.len());
            let rt_handle = self.rt_handle.clone();
            
            // 使用Arc包装事件，确保所有观察者都能访问到事件
            let event_arc = std::sync::Arc::new(event);
            
            for observer in obs {
                let observer_clone = observer.clone();
                let event_clone = event_arc.clone();
                
                rt_handle.spawn(async move {
                    // 获取E类型的引用，而不是Arc<E>类型的引用
                    let event_ref: &dyn Any = &*event_clone;
                    on_event(&*observer_clone, event_ref);
                });
            }
        } else {
            println!("No observers found for event type: {:?}", event_type_id);
        }
    }
}