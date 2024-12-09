use axum::Router;

pub struct API {
    router: Router,
}

impl API {
    pub fn new() -> Self {
        let router = Router::new();

        API { router }
    }

    pub fn into_router(self) -> Router {
        self.router
    }
}
