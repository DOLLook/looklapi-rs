use axum::{Extension, Router, routing::get, routing::post};
use inventory::submit;

use crate::{
    app::{AppError, AppResponse},
    controller::Controller,
};

/// 用户控制器
struct UserController;

impl Controller for UserController {
    fn routes() -> Router {
        Router::new()
            .route("/user/info", get(get_user_info))
            // 这里可以为特定路由添加中间件，实现AOP效果
            .route(
                "/user/protected",
                get(protected_handler), // 可以在这里添加登录验证中间件
                                        // .layer(axum::middleware::from_fn(controller::middleware::auth_middleware))
            )
    }
}

// 注册控制器
// submit! {
//     crate::controller::registry::ControllerRegistration {
//         routes: UserController::routes,
//     }
// }
crate::register_controller!(UserController);

async fn get_user_info() -> Result<AppResponse<&'static str>, AppError> {
    Ok(AppResponse::new("User info endpoint"))
}

async fn protected_handler(
    Extension(ctx): Extension<crate::request_context::RequestContext>,
) -> Result<AppResponse<&'static str>, AppError> {
    Ok(AppResponse::new("Protected endpoint"))
}
