CREATE INDEX idx_created_at_date ON messages (created_at);

CREATE TYPE message_type_enum AS ENUM ('text', 'image', 'video', 'audio');

ALTER TABLE messages
ADD COLUMN message_type message_type_enum;

CREATE INDEX idx_message_type_bitmap ON messages  (message_type);

ALTER TYPE message_type_enum ADD VALUE 'document'