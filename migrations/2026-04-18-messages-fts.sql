-- FTS5 index over messages with the `trigram` tokenizer, giving indexed
-- substring search (3 chars or more) across account_name, character_name, and
-- text. The table is `content=`-linked to `messages` so rows are not
-- duplicated; triggers keep the index in sync on INSERT/UPDATE/DELETE.

CREATE VIRTUAL TABLE messages_fts USING fts5(
    account_name,
    character_name,
    text,
    content='messages',
    content_rowid='rowid',
    tokenize='trigram'
);

INSERT INTO messages_fts(rowid, account_name, character_name, text)
    SELECT rowid, account_name, character_name, text FROM messages;

CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, account_name, character_name, text)
    VALUES (new.rowid, new.account_name, new.character_name, new.text);
END;

CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, account_name, character_name, text)
    VALUES ('delete', old.rowid, old.account_name, old.character_name, old.text);
END;

CREATE TRIGGER messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, account_name, character_name, text)
    VALUES ('delete', old.rowid, old.account_name, old.character_name, old.text);
    INSERT INTO messages_fts(rowid, account_name, character_name, text)
    VALUES (new.rowid, new.account_name, new.character_name, new.text);
END;
