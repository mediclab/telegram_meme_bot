use crate::database::entity::messages::EntityTypes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CallbackOperations {
    AddToFButton,
    AddToUserLeft,
    AddToNewbieUser,
    AddToSimilarMeme,
    AddToMemeAlreadyExists,
    Cancel,
}

impl From<CallbackOperations> for EntityTypes {
    fn from(value: CallbackOperations) -> Self {
        match value {
            CallbackOperations::AddToFButton => EntityTypes::PressFToPrayRespects,
            CallbackOperations::AddToUserLeft => EntityTypes::UserLeftChat,
            CallbackOperations::AddToNewbieUser => EntityTypes::NewbieUser,
            CallbackOperations::AddToSimilarMeme => EntityTypes::SimilarMeme,
            CallbackOperations::AddToMemeAlreadyExists => EntityTypes::MemeAlreadyExists,
            _ => {
                unreachable!()
            }
        }
    }
}
