CREATE TABLE user (
    id          TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    updated_at  TEXT,

    PRIMARY KEY (id)
);

CREATE TABLE favorites (
    id          TEXT    NOT NULL,
    user_id     TEXT    NOT NULL,
    guild_id    TEXT    NOT NULL,
    title       TEXT    NOT NULL,
    uri         TEXT    NOT NULL,

    created_at  TEXT    NOT NULL,
    updated_at  TEXT,

    PRIMARY KEY (id)
);