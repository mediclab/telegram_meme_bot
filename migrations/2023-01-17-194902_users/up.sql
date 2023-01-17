CREATE TABLE IF NOT EXISTS users
(
    user_id BIGSERIAL,
    username VARCHAR(33),
    firstname VARCHAR(65) NOT NULL,
    lastname VARCHAR(65),
    bio VARCHAR(71),
    deleted_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);