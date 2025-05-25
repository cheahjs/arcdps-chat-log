CREATE INDEX IF NOT EXISTS messages_account_name_idx ON messages (account_name);
CREATE INDEX IF NOT EXISTS messages_character_name_idx ON messages (character_name);
CREATE INDEX IF NOT EXISTS messages_text_idx ON messages (text);
