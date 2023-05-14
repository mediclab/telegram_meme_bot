CREATE TABLE IF NOT EXISTS chat_admins
(
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chat_id BIGSERIAL NOT NULL,
    user_id BIGSERIAL NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX ON chat_admins (chat_id, user_id);