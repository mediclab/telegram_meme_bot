CREATE TABLE IF NOT EXISTS chats
(
    chat_id BIGSERIAL,
    chatname VARCHAR(256),
    description VARCHAR(256),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);