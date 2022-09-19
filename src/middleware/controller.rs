use std::collections::HashMap;

use crate::Masking;

use super::RequestId;

#[derive(Debug)]
pub(crate) enum Message {
    WithMasking {
        request_id: RequestId,
        masking: Masking,
    },
}

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

    pub(crate) fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::WithMasking {
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

pub struct Controller {}
