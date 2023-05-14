CREATE TABLE IF NOT EXISTS messages
(
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_type VARCHAR(256) NOT NULL,
    message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);