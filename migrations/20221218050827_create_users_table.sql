-- Add migration script here
CREATE TABLE users(
    user_id uuid PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL
)
