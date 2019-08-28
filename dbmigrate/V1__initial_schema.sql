CREATE TABLE api_key (
    id uuid UNIQUE PRIMARY KEY,
    email text NOT NULL,
    hashed_key text NOT NULL
);

CREATE TABLE inc_sequence (
    id uuid UNIQUE PRIMARY KEY,
    owner_id uuid NOT NULL REFERENCES api_key(id),
    seq_value bigint NOT NULL
);
