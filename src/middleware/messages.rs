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
#[allow(clippy::enum_variant_names)]
pub enum ControllerMessage {
    SetMasking {
        request_id: RequestId,
        masking: Box<Masking>,
    },
    SetPathHint {
        request_id: RequestId,
        path_hint: String,
    },
    SetCustomerId {
        request_id: RequestId,
        customer_id: String,
    },
    SetMaxCaptureSize {
        request_id: RequestId,
        capture_size: u64,
    },
}
