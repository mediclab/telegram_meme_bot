// @generated automatically by Diesel CLI.

diesel::table! {
    meme_likes (id) {
        id -> Int8,
        user_id -> Int8,
        meme_id -> Int8,
        num -> Nullable<Int2>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    memes (id) {
        id -> Int8,
        user_id -> Int8,
        chat_id -> Int8,
        photos -> Nullable<Jsonb>,
        posted_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    meme_likes,
    memes,
);
