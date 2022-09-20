use std::collections::HashMap;

use crate::{
    middleware::{
        messages::{ControllerMessage, MiddlewareMessage},
        RequestId,
    },
    path_hint, Masking,
};

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
            ControllerMessage::SetMasking {
                request_id,
                masking,
            } => {
                self.masks.insert(request_id, *masking);
            }
            ControllerMessage::SetPathHint {
                request_id,
                path_hint,
            } => {
                self.path_hints.insert(request_id, path_hint);
            }
            ControllerMessage::SetCustomerId {
                request_id,
                customer_id,
            } => {
                self.customer_ids.insert(request_id, customer_id);
            }
        }
    }

    pub(crate) fn get_masking(&mut self, request_id: &RequestId) -> Option<Masking> {
        self.masks.remove(request_id)
    }
}

#[derive(Debug, Clone)]
pub struct Controller {
    request_id: RequestId,
    sender: Sender<MiddlewareMessage>,
}

impl Controller {
    pub fn new(request_id: RequestId, sender: Sender<MiddlewareMessage>) -> Self {
        Self { request_id, sender }
    }

    pub async fn set_path_hint(&self, path_hint: &str) {
        let path_hint = path_hint::normalize(path_hint);

        self.sender
            .clone()
            .send(MiddlewareMessage::ControllerMessage(
                ControllerMessage::SetPathHint {
                    request_id: self.request_id.clone(),
                    path_hint,
                },
            ))
            .await
            .unwrap();
    }

    pub async fn set_masking(&self, masking: Masking) {
        self.sender
            .clone()
            .send(MiddlewareMessage::ControllerMessage(
                ControllerMessage::SetMasking {
                    request_id: self.request_id.clone(),
                    masking: Box::new(masking),
                },
            ))
            .await
            .unwrap();
    }

    pub async fn set_customer_id(&self, customer_id: String) {
        self.sender
            .clone()
            .send(MiddlewareMessage::ControllerMessage(
                ControllerMessage::SetCustomerId {
                    request_id: self.request_id.clone(),
                    customer_id,
                },
            ))
            .await
            .unwrap();
    }
}

// private methods used in middleware
#[doc(hidden)]
impl Controller {
    pub(crate) fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    pub(crate) fn sender(&self) -> &Sender<MiddlewareMessage> {
        &self.sender
    }
}
