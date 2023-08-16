// @generated automatically by Diesel CLI.

diesel::table! {
    chat_admins (uuid) {
        uuid -> Uuid,
        chat_id -> Int8,
        user_id -> Int8,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    chats (chat_id) {
        chat_id -> Int8,
        #[max_length = 256]
        chatname -> Nullable<Varchar>,
        #[max_length = 256]
        description -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        #[max_length = 256]
        title -> Nullable<Varchar>,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    meme_likes (uuid) {
        uuid -> Uuid,
        meme_uuid -> Nullable<Uuid>,
        user_id -> Int8,
        num -> Int2,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    memes (uuid) {
        uuid -> Uuid,
        msg_id -> Nullable<Int8>,
        user_id -> Int8,
        chat_id -> Int8,
        photos -> Nullable<Jsonb>,
        posted_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        #[max_length = 256]
        long_hash -> Nullable<Varchar>,
        #[max_length = 4]
        short_hash -> Nullable<Varchar>,
    }
}

diesel::table! {
    messages (uuid) {
        uuid -> Uuid,
        #[max_length = 256]
        message_type -> Varchar,
        message -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Int8,
        #[max_length = 33]
        username -> Nullable<Varchar>,
        #[max_length = 65]
        firstname -> Varchar,
        #[max_length = 65]
        lastname -> Nullable<Varchar>,
        deleted_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(meme_likes -> memes (meme_uuid));

diesel::allow_tables_to_appear_in_same_query!(
    chat_admins,
    chats,
    meme_likes,
    memes,
    messages,
    users,
);
