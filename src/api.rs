use axum::{
    routing::post,
    Router,
};

use crate::exec::Exec;

pub struct Api {
    router: Router,
}

impl Api {
    pub fn new() -> Self {
        let router = Router::new().route(
            "/exec",
            post(|req| async { Exec::new().execute(req).await }),
        );

        Api { router }
    }

    pub fn into_router(self) -> Router {
        self.router
    }
}
