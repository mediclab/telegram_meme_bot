CREATE TABLE IF NOT EXISTS meme_likes (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id BIGINT NOT NULL,
    msg_id BIGINT NOT NULL,
    num SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX ON meme_likes (user_id, msg_id);