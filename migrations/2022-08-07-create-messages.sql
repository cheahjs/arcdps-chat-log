CREATE TABLE messages(
                channel_id      INTEGER     NOT NULL,
                channel_type    INTEGER     NOT NULL,
                subgroup        INTEGER     NOT NULL,
                is_broadcast    BOOLEAN     NOT NULL,
                timestamp       INTEGER     NOT NULL,
                account_name    TEXT        NOT NULL,
                character_name  TEXT        NOT NULL,
                text            TEXT        NOT NULL,
                game_start      INTEGER     NOT NULL
            );