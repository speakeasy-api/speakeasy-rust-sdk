use crate::{
    generic_http::{GenericRequest, GenericResponse},
    Masking,
};

use super::RequestId;

#[doc(hidden)]
#[derive(Debug)]
pub enum MiddlewareMessage {
    Request(RequestMessage),
    Response(ResponseMessage),
    ControllerMessage(ControllerMessage),
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RequestMessage {
    pub(crate) request_id: RequestId,
    pub(crate) request: GenericRequest,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ResponseMessage {
    pub(crate) request_id: RequestId,
    pub(crate) response: GenericResponse,
}

#[derive(Debug)]
pub enum ControllerMessage {
    WithMasking {
        request_id: RequestId,
        masking: Masking,
    },
}
