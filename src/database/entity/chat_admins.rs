use crate::database::Database;
use futures::{executor::block_on, future::join_all};
use sea_orm::{entity::prelude::*, QuerySelect, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "chat_admins")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub chat_id: i64,
    pub user_id: i64,
    pub created_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn add_admins(chat_id: i64, admins_ids: &[u64]) {
        let futures: Vec<_> = admins_ids
            .iter()
            .map(|admin_id| {
                Entity::insert(ActiveModel {
                    chat_id: Set(chat_id),
                    user_id: Set(*admin_id as i64),
                    ..Default::default()
                })
                .exec(Database::global().connection())
            })
            .collect();

        block_on(join_all(futures));
    }

    pub async fn get_admin_chats(user_id: u64) -> Vec<i64> {
        let res = Self::find()
            .filter(Column::UserId.eq(user_id as i64))
            .select_only()
            .column(Column::ChatId)
            .into_tuple()
            .all(Database::global().connection())
            .await;

        res.unwrap_or_else(|e| {
            error!("Can't get memes by short hash from database: {e}");
            Vec::new()
        })
    }

    pub fn is_user_admin(user_id: i64) -> bool {
        let query = Self::find()
            .filter(Column::UserId.eq(user_id))
            .one(Database::global().connection());

        match block_on(query) {
            Ok(m) => m.is_some(),
            Err(_) => false,
        }
    }
}
