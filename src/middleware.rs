mod controller;
mod request_id;

use crate::generic_http::{GenericRequest, GenericResponse};

// 1MB
pub(crate) const MAX_SIZE: usize = 1024 * 1024;

pub(crate) type RequestId = request_id::RequestId;

#[derive(Debug)]
pub(crate) enum MiddlewareMessage {
    Request {
        request_id: RequestId,
        request: GenericRequest,
    },
    Response {
        request_id: RequestId,
        response: GenericResponse,
    },
}

// framework specific middleware
#[cfg(feature = "actix3")]
pub mod actix3;
