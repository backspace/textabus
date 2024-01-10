CREATE TABLE NUMBERS (
    number VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255),
    approved BOOLEAN DEFAULT false,
    admin BOOLEAN DEFAULT false,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);