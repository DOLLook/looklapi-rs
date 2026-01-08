use axum::Router;
use inventory;

/// 控制器注册项，用于在静态上下文中存储路由工厂
pub struct ControllerRegistration {
    pub routes: fn() -> Router,
}

// 为ControllerRegistration实现inventory的Collect trait
inventory::collect!(ControllerRegistration);

/// 从inventory中收集所有控制器的路由
pub fn collect_routes() -> Router {
    let mut app = Router::new();

    // 遍历所有注册的控制器并合并路由
    for registration in inventory::iter::<ControllerRegistration>() {
        app = app.merge((registration.routes)());
    }

    app
}
