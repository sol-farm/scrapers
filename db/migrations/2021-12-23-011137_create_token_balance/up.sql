CREATE TABLE token_balance (
    id BIGSERIAL PRIMARY KEY,
    token_account VARCHAR NOT NULL,
    token_mint VARCHAR NOT NULL,
    -- an identifier used to describe this balance
    identifier VARCHAR NOT NULL,
    balance FLOAT8 NOT NULL DEFAULT 0,
    scraped_at TIMESTAMPTZ NOT NULL
);