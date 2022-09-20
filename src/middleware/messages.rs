use crate::{
    generic_http::{GenericRequest, GenericResponse},
    Masking,
};

use super::RequestId;

#[doc(hidden)]
#[derive(Debug)]
pub enum MiddlewareMessage {
    Request {
        request_id: RequestId,
        request: GenericRequest,
    },
    Response {
        request_id: RequestId,
        response: GenericResponse,
    },
    ControllerMessage(ControllerMessage),
}

#[derive(Debug)]
pub enum ControllerMessage {
    SetMasking {
        request_id: RequestId,
        masking: Box<Masking>,
    },
    SetPathHint {
        request_id: RequestId,
        path_hint: String,
    },
}
