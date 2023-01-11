CREATE TABLE notes(
    account_name    TEXT    PRIMARY KEY,
    note            TEXT    NOT NULL,
    note_added      INTEGER NOT NULL,
    note_updated    INTEGER NOT NULL,
    last_seen       INTEGER NOT NULL,
);