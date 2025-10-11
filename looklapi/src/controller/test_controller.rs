use looklapi_macro::proxy;

use crate::app::AppError;

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
