-- does not track stats related to the obligation, simply provides a database
-- of all obligation accounts that exist
CREATE TABLE v1_obligation_account (
  id BIGSERIAL PRIMARY KEY,
  -- the obligation account address
  account VARCHAR NOT NULL UNIQUE,
  -- the owner of the obligation account, usually a user farm
  authority VARCHAR NOT NULL
);

ALTER TABLE token_price ADD COLUMN period_start TIMESTAMPTZ NOT NULL;
ALTER TABLE token_price ADD COLUMN period_end TIMESTAMPTZ NOT NULL;
ALTER TABLE token_price ADD COLUMN period_observed_prices FLOAT8[] NOT NULL DEFAULT '{}';
ALTER TABLE token_price ADD COLUMN period_running_average FLOAT8 NOT NULL DEFAULT 0;
ALTER TABLE token_price ADD COLUMN last_period_average FLOAT8 NOT NULL DEFAULT 0;
ALTER TABLE token_price ADD COLUMN feed_stopped BOOLEAN NOT NULL DEFAULT false;
-- mint of the token address
ALTER TABLE token_price ADD COLUMN token_mint VARCHAR NOT NULL DEFAULT '';