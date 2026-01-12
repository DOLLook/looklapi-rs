use axum::{Extension, Router, body::Body, http::Request, response::IntoResponse, routing::get};
use base64::Engine as _;
use base64::engine::general_purpose;
use inventory::submit;

use crate::{
    app::{AppError, AppResponse},
    commonapi::drawer::{new_image_drawer},
    controller::Controller,
    model::modelimpl::draw::{ImageContentModel, LineContentModel, RectangleContentModel, TextModel},
    request_context::{self, X_REQUEST_ID},
};

/// 测试控制器
struct TestController;

impl Controller for TestController {
    fn routes() -> Router {
        Router::new()
            .route("/test", get(test_handler))
            .route("/test/hello", get(hello_handler))
            .route(
                "/test/handler",
                get(handler)
                    .layer(axum::middleware::from_fn(test_after))
                    .layer(axum::middleware::from_fn(test_begin)),
            )
            .route("/test/draw", get(draw_handler))
    }
}

// 注册控制器
// submit! {
//     crate::controller::registry::ControllerRegistration {
//         routes: TestController::routes,
//     }
// }
crate::register_controller!(TestController);

async fn test_handler(
    Extension(ctx): Extension<crate::request_context::RequestContext>,
) -> Result<AppResponse<&'static str>, AppError> {
    println!("Test controller handler called");
    Ok(AppResponse::new("Test endpoint"))
}

async fn hello_handler() -> Result<AppResponse<&'static str>, AppError> {
    Ok(AppResponse::new("Hello from test controller"))
}

trait TestTrait {
    fn proxy_fn(&self) -> Result<(), AppError>;
    fn other_fn(&self);
}

struct Test;

impl TestTrait for Test {
    fn proxy_fn(&self) -> Result<(), AppError> {
        Err(AppError::new("test error"))
    }

    fn other_fn(&self) {
        todo!()
    }
}

// #[proxy(TestTrait)]
struct TestProxy {
    inner: Test,
}

impl TestProxy {
    fn proxy_fn(&self) -> Result<(), AppError> {
        println!("begin test");
        let result = self.inner.proxy_fn();
        println!("after test");
        result
    }

    fn other_fn(&self) {
        self.inner.other_fn();
    }
}

async fn handler(
    Extension(ctx): Extension<request_context::RequestContext>,
) -> Result<AppResponse<i32>, AppError> {
    for (k, v) in ctx.header.iter() {
        println!("{}: {}", k, v.to_str().unwrap_or("few"));
    }
    // panic!("panic123455");
    try_thing()?;
    Ok(AppResponse::new(123))
}

fn try_thing() -> Result<(), anyhow::Error> {
    anyhow::bail!("it failed!")
}

async fn test_begin(mut req: Request<Body>, next: axum::middleware::Next) -> impl IntoResponse {
    let req_id = req.headers().get(X_REQUEST_ID).unwrap().to_str().unwrap();

    println!("test_begin请求ID: {}", req_id);

    next.run(req).await
}

async fn test_after(mut req: Request<Body>, next: axum::middleware::Next) -> impl IntoResponse {
    let req_id = req
        .headers()
        .get(X_REQUEST_ID)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let rsp = next.run(req).await;
    println!("test_after请求ID: {}", req_id);
    rsp
}

async fn draw_handler() -> Result<AppResponse<String>, AppError> {
    // 创建绘图实例
    let mut drawer = new_image_drawer();
    
    // 初始化绘图上下文
    drawer.init(800, 600)?;
    
    // 创建背景模板
    let template = ImageContentModel {
        rectangle_content_model: RectangleContentModel {
            draw_content_model: crate::model::modelimpl::draw::DrawContentModel {
                agb_color: vec![240, 240, 240], // 浅灰色背景
                x: 0,
                y: 0,
            },
            width: 800,
            height: 600,
            radius: 0,
            transparent: 255,
        },
        serial: 0,
        path: String::new(),
    };
    
    // 绘制背景模板
    drawer.draw_template(template)?;
    
    // 绘制矩形
    let rect = RectangleContentModel {
        draw_content_model: crate::model::modelimpl::draw::DrawContentModel {
            agb_color: vec![0, 128, 255], // 蓝色
            x: 100,
            y: 100,
        },
        width: 600,
        height: 400,
        radius: 20,
        transparent: 200,
    };
    drawer.draw_rectangle(rect)?;
    
    // 绘制线条
    let line = LineContentModel {
        draw_content_model: crate::model::modelimpl::draw::DrawContentModel {
            agb_color: vec![255, 0, 0], // 红色
            x: 100,
            y: 300,
        },
        width: 5,
        length: 600,
    };
    drawer.draw_line(line)?;
    
    // 绘制文字
    let text = TextModel {
        draw_content_model: crate::model::modelimpl::draw::DrawContentModel {
            agb_color: vec![0, 0, 0], // 黑色
            x: 400,
            y: 200,
        },
        is_print: true,
        text: "Hello, Rust Drawer!".to_string(),
        font: "Arial".to_string(),
        size: 36.0,
        is_bold: true,
        is_italic: false,
        h_gap: 0.0,
        v_gap: 0.0,
        alignment: 1, // 居中对齐
        boundary: 800,
        row_limit: 1,
        related_dynamic_text: Vec::new(),
    };
    drawer.draw_text(text)?;
    
    // 生成图片字节流
    let bytes = drawer.encode_to_bytes()?;
    
    // 将字节流转换为base64
    let base64_str = general_purpose::STANDARD.encode(bytes);
    let data_url = format!("data:image/png;base64,{}", base64_str);
    
    Ok(AppResponse::new(data_url))
}
