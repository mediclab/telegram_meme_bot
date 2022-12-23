// @generated automatically by Diesel CLI.

diesel::table! {
    meme_likes (id) {
        id -> Int8,
        user_id -> Int8,
        msg_id -> Int8,
        num -> Nullable<Int2>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    memes (msg_id) {
        msg_id -> Int8,
        bot_msg_id -> Int8,
        user_id -> Int8,
        chat_id -> Int8,
        photos -> Nullable<Jsonb>,
        posted_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(meme_likes -> memes (msg_id));

diesel::allow_tables_to_appear_in_same_query!(
    meme_likes,
    memes,
);
