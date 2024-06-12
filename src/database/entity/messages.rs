use crate::database::Database;
use rand::prelude::SliceRandom;
use sea_orm::{entity::prelude::*, ActiveValue::Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub r#type: MessageTypes,
    pub entity_type: EntityTypes,
    #[sea_orm(column_type = "Text", nullable)]
    pub message: String,
    pub created_at: Option<DateTime>,
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = r#""MessageType""#)]
pub enum MessageTypes {
    #[sea_orm(string_value = "text")]
    Text,
    #[sea_orm(string_value = "photo")]
    Photo,
    #[sea_orm(string_value = "video")]
    Video,
    #[sea_orm(string_value = "document")]
    Document,
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = r#""MessageEntityType""#)]
pub enum EntityTypes {
    #[sea_orm(string_value = "meme_already_exists")]
    MemeAlreadyExists,
    #[sea_orm(string_value = "similar_meme")]
    SimilarMeme,
    #[sea_orm(string_value = "newbie_user")]
    NewbieUser,
    #[sea_orm(string_value = "user_left_chat")]
    UserLeftChat,
    #[sea_orm(string_value = "press_f_to_pray_respects")]
    PressFToPrayRespects,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn add(message_type: MessageTypes, entity_type: EntityTypes, text: &str) -> bool {
        Entity::insert(ActiveModel {
            r#type: Set(message_type),
            entity_type: Set(entity_type),
            message: Set(text.to_string()),
            ..Default::default()
        })
        .exec(Database::global().connection())
        .await
        .is_ok()
    }

    pub async fn get_random_text(entity_type: EntityTypes) -> String {
        if let Some(model) = Self::get_random(MessageTypes::Text, entity_type).await {
            model.message
        } else {
            String::new()
        }
    }

    pub async fn get_random_photo(entity_type: EntityTypes) -> String {
        if let Some(model) = Self::get_random(MessageTypes::Photo, entity_type).await {
            model.message
        } else {
            String::new()
        }
    }

    pub async fn get_random(message_type: MessageTypes, entity_type: EntityTypes) -> Option<Model> {
        let res = Self::find()
            .filter(Column::Type.eq(message_type))
            .filter(Column::EntityType.eq(entity_type))
            .all(Database::global().connection())
            .await;

        match res {
            Ok(m) => m.choose(&mut rand::thread_rng()).cloned(),
            Err(e) => {
                error!("Can't get texts from database: {e}");
                None
            }
        }
    }
}
