-- Включение расширения для генерации UUID
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Таблица chats
CREATE TABLE chats (
    uid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Таблица chat_participants
CREATE TABLE chat_participants (
    chat_uid UUID NOT NULL,
    user_uid UUID NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (chat_uid, user_uid),
    FOREIGN KEY (chat_uid) REFERENCES chats(uid) ON DELETE CASCADE
);

-- Таблица messages
CREATE TABLE messages (
    uid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chat_uid UUID NOT NULL,
    user_uid UUID NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (chat_uid) REFERENCES chats(uid) ON DELETE CASCADE
);