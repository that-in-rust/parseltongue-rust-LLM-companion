-- Simple SQL schema for testing v1.5.6 SQL language support
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE VIEW active_users AS
SELECT * FROM users
WHERE created_at > CURRENT_DATE - INTERVAL '30 days';
