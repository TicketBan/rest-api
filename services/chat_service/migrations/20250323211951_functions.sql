CREATE OR REPLACE FUNCTION update_chat_timestamp_on_participant_add() RETURNS TRIGGER AS $$
BEGIN
  UPDATE chats
  SET updated_at = CURRENT_TIMESTAMP
  WHERE uid = NEW.chat_uid;
  
  RETURN NEW;  
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_chat_timestamp_on_message_add() RETURNS TRIGGER AS $$
BEGIN
  UPDATE chats
  SET updated_at = CURRENT_TIMESTAMP
  WHERE uid = NEW.chat_uid;
  
  RETURN NEW;  
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_chat_on_participant_add
AFTER INSERT ON chat_participants
FOR EACH ROW
EXECUTE FUNCTION update_chat_timestamp_on_participant_add();

CREATE TRIGGER update_chat_on_message_add
AFTER INSERT ON messages
FOR EACH ROW
EXECUTE FUNCTION update_chat_timestamp_on_message_add();

