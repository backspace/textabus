CREATE TABLE api_responses (
    id UUID PRIMARY KEY,
    body TEXT NOT NULL,
    query TEXT NOT NULL,
    message_id UUID REFERENCES messages(id),
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);