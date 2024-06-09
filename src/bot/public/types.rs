use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum CallbackOperations {
    Like,
    Dislike,
    Delete,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemeCallback {
    pub uuid: Uuid,
    pub op: CallbackOperations,
}
