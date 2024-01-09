CREATE TABLE messages (
    id UUID PRIMARY KEY,
    message_sid VARCHAR(34),
    origin VARCHAR(255) NOT NULL,
    destination VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    initial_message_id UUID references messages(id),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);