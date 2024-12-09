use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use derive_setters::Setters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct Cause {
    error: String,
    cause: Option<Box<Cause>>,
}

impl IntoResponse for Cause {
    fn into_response(self) -> Response {
        Response::new(Body::from(serde_json::to_string(&self).unwrap()))
    }
}

impl Cause {
    pub fn new(error: impl Into<String>) -> Cause {
        Cause {
            error: error.into(),
            cause: None,
        }
    }
}
