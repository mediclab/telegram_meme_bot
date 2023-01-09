CREATE TABLE IF NOT EXISTS memes
(
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    msg_id BIGINT NULL,
    user_id BIGINT NOT NULL,
    chat_id BIGINT NOT NULL,
    photos JSONB NULL,
    posted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON memes (msg_id);
CREATE INDEX ON memes (user_id, chat_id);