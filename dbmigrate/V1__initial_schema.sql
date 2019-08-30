CREATE TABLE api_key (
    id uuid UNIQUE PRIMARY KEY,
    email text NOT NULL,
    prefix bytea NOT NULL,
    hashed_key bytea NOT NULL,
    created_dt TIMESTAMP NOT NULL,
    UNIQUE (prefix, hashed_key)
);

CREATE TABLE inc_sequence (
    id uuid UNIQUE PRIMARY KEY,
    owner_id uuid NOT NULL REFERENCES api_key(id),
    seq_value bigint NOT NULL
);
