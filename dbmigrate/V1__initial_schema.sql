CREATE TABLE api_key (
    id uuid UNIQUE PRIMARY KEY,
    email text UNIQUE NOT NULL,
    prefix bytea NOT NULL,
    hashed_key bytea NOT NULL,
    created_dt TIMESTAMP NOT NULL,
    UNIQUE (prefix, hashed_key)
);

CREATE TABLE counter (
    id uuid UNIQUE PRIMARY KEY,
    owner_id uuid NOT NULL REFERENCES api_key(id),
    tag text NOT NULL,
    counter_value bigint NOT NULL,
    last_updated TIMESTAMP,
    UNIQUE (owner_id, tag)
);
