ALTER TABLE deposit_tracking ADD COLUMN current_balance FLOAT8 NOT NULL DEFAULT 0;
ALTER TABLE deposit_tracking ADD COLUMN current_shares FLOAT8 NOT NULL DEFAULT 0;
ALTER TABLE deposit_tracking ADD COLUMN balance_usd_value FLOAT8 NOT NULL DEFAULT 0;