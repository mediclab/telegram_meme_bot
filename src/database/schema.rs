// @generated automatically by Diesel CLI.

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
    }
}

diesel::joinable!(meme_likes -> memes (meme_uuid));

diesel::allow_tables_to_appear_in_same_query!(meme_likes, memes,);
