use axum::{Router, http::Method};
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use tracing::info;

use crate::app::app_config;

mod app;
mod common;
mod commonapi;
mod controller;
mod model;
mod request_context;

#[tokio::main]
async fn main() {
    // let mut ctx = rudi::Context::auto_register();
    // let app_config = ctx.resolve::<app_config::AppConfig>();
    let ctx = rudi::Context::options().eager_create(true).auto_register();

    let app_config = ctx.get_single::<app_config::AppConfig>();
    common::loggers::init_logger(app_config).await;

    app::appcontext::publisher::get_app_event_publisher()
        .publish_event(app::appcontext::events::AppEventBeanInjected);

    let app = app();

    // AppError::new("here is a error").log();

    let listen = format!("0.0.0.0:{}", app_config.server.port);
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    // info!("app start 完成");
    axum::serve(listener, app).await.unwrap();
}

fn app() -> Router {
    // let request_id_middleware =
    //     ServiceBuilder::new().layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
    // send headers from request to response headers
    // .layer(PropagateRequestIdLayer::new(x_request_id));
    Router::new()
        // .route(
        //     "/",
        //     get(handler)
        //         .layer(axum::middleware::from_fn(test_after))
        //         .layer(axum::middleware::from_fn(test_begin)),
        // )
        // 合并所有控制器的路由
        .merge(controller::collect_routes())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::any())
                .allow_headers(AllowHeaders::any())
                .allow_methods([Method::OPTIONS, Method::HEAD, Method::GET, Method::POST]),
        )
        .layer(axum::middleware::from_fn(
            controller::middleware::request_ctx_middleware,
        ))
        .layer(axum::middleware::from_fn(
            controller::middleware::request_id_middleware,
        ))
        .layer(axum::middleware::from_fn(
            controller::middleware::panic_handler,
        ))
    // .layer(request_id_middleware)
}
