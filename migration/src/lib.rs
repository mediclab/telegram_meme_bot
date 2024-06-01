pub use sea_orm_migration::prelude::*;

mod m20240524_200516_create_memes_table;
mod m20240524_200524_create_meme_likes_table;
mod m20240524_200536_create_users_table;
mod m20240524_200553_create_chats_table;
mod m20240524_200600_create_messages_table;
mod m20240524_200614_create_chat_admins_table;
mod m20240531_121814_modify_messages_table;
mod m20240531_143248_insert_data_to_messages;
mod m20240602_073326_add_foreign_keys;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240524_200516_create_memes_table::Migration),
            Box::new(m20240524_200524_create_meme_likes_table::Migration),
            Box::new(m20240524_200536_create_users_table::Migration),
            Box::new(m20240524_200553_create_chats_table::Migration),
            Box::new(m20240524_200600_create_messages_table::Migration),
            Box::new(m20240524_200614_create_chat_admins_table::Migration),
            Box::new(m20240531_121814_modify_messages_table::Migration),
            Box::new(m20240531_143248_insert_data_to_messages::Migration),
            Box::new(m20240602_073326_add_foreign_keys::Migration),
        ]
    }
}
