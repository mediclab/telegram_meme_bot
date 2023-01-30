// @generated automatically by Diesel CLI.

diesel::table! {
    chats (chat_id) {
        chat_id -> Int8,
        chatname -> Varchar,
        description -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
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
        long_hash -> Nullable<Varchar>,
        short_hash -> Nullable<Varchar>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Int8,
        username -> Nullable<Varchar>,
        firstname -> Varchar,
        lastname -> Nullable<Varchar>,
        deleted_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(meme_likes -> memes (meme_uuid));

diesel::allow_tables_to_appear_in_same_query!(chats, meme_likes, memes, users,);
