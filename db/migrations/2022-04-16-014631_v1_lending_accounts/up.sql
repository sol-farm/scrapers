-- Your SQL goes here

-- the ltv of v1 lending obligations
CREATE TABLE v1_obligation_ltv (
  id BIGSERIAL PRIMARY KEY,
  -- the wallet/user that manages the user farm which manages the obligations
  authority VARCHAR NOT NULL,
  -- the userfarm which this obligation belongs to
  user_farm VARCHAR NOT NULL,
  account_address VARCHAR NOT NULL unique,
  ltv FLOAT8 NOT NULL DEFAULT 0,
  scraped_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE v1_user_farm (
    id BIGSERIAL PRIMARY KEY,
    account_address VARCHAR NOT NULL UNIQUE,
    authority VARCHAR NOT NULL,
    obligations TEXT[] NOT NULL DEFAULT '{}',
    obligation_indexes INTEGER[] NOT NULL DEFAULT '{}'
);


CREATE TABLE v1_liquidated_position (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    -- a unique identifier associated with a liquidation event
    liquidation_event_id VARCHAR NOT NULL UNIQUE,
    -- the temp liquidation account that was used to
    -- store information related to the liquidation process
    temp_liquidation_account VARCHAR NOT NULL,
    -- the user/wallet that owns the position
    authority VARCHAR NOT NULL,
    -- the userfarm which manages multiple positions
    user_farm VARCHAR NOT NULL,
    -- the actual position which was liquidated
    obligation VARCHAR NOT NULL,
    -- the time at which the position started to be liquidated
    started_at TIMESTAMPTZ NOT NULL,
    -- the time at which the position was finally liquidated
    ended_at TIMESTAMPTZ
); 