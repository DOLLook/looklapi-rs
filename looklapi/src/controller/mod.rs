use axum::Router;

pub mod middleware;
mod registry;
mod test_controller;
mod user_controller;

/// 控制器 trait，所有控制器都需要实现此 trait
trait Controller {
    /// 返回控制器的路由
    fn routes() -> Router;
}

/// 注册控制器的宏
#[macro_export]
macro_rules! register_controller {
    ($controller:ty) => {
        inventory::submit! {
            $crate::controller::registry::ControllerRegistration {
                routes: <$controller>::routes,
            }
        }
    };
}

/// 自动收集所有控制器的路由
pub fn collect_routes() -> Router {
    registry::collect_routes()
}
