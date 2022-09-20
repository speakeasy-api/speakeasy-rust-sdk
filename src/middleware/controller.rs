use std::collections::HashMap;

use super::{
    messages::{ControllerMessage, MiddlewareMessage},
    RequestId,
};
use crate::Masking;

use tokio02::sync::mpsc::Sender;

#[derive(Debug)]
pub struct State {
    customer_ids: HashMap<RequestId, String>,
    path_hints: HashMap<RequestId, String>,
    masks: HashMap<RequestId, Masking>,
}

impl State {
    pub fn new() -> Self {
        Self {
            customer_ids: HashMap::new(),
            path_hints: HashMap::new(),
            masks: HashMap::new(),
        }
    }

    pub(crate) fn handle_message(&mut self, msg: ControllerMessage) {
        match msg {
            ControllerMessage::WithMasking {
                request_id,
                masking,
            } => {
                self.masks.insert(request_id, masking);
            }
        }
    }

    pub(crate) fn get_masking(&mut self, request_id: &RequestId) -> Option<Masking> {
        self.masks.remove(request_id)
    }
}

#[derive(Debug)]
pub struct Controller {
    request_id: RequestId,
    sender: Sender<MiddlewareMessage>,
}

impl Controller {
    pub fn new(request_id: RequestId, sender: Sender<MiddlewareMessage>) -> Self {
        Self { request_id, sender }
    }

    pub fn set_path_hint(&self) {}
}
