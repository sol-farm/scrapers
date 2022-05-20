-- Your SQL goes here
-- for information on using this with grafana see https://github.com/grafana/piechart-panel/issues/142

CREATE TABLE lending_optimizer_distribution (
  id BIGSERIAL PRIMARY KEY,
  -- the combination of farm name + tag
  -- LENDING-USDC with a tag of sollend would be `LENDING-USDC-tag(solend)`
  -- we add a unique constraint so we only maintain one entry of this
  vault_name VARCHAR NOT NULL UNIQUE,
  -- the platform name of the standalone vault
  -- this is sorted to match the way standalone vaults are
  -- listed in the vault account itself
  -- standalone_vault_platforms[0] corresponds to standalone_vault_deposited_balance[0]
  standalone_vault_platforms TEXT[] NOT NULL,
  -- the amount of funds deposited into the vault
  standalone_vault_deposited_balances FLOAT8[] NOT NULL
);