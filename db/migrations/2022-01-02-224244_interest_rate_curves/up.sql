-- Your SQL goes here

CREATE TABLE interest_rate_curve (
  id BIGSERIAL PRIMARY KEY,
  -- platform name
  platform VARCHAR NOT NULL,
  -- the asset
  asset VARCHAR NOT NULL,
  -- unique name, combination of PLATFORM-ASSET in uppercase
  -- ensure we dont have multiple curves for the same platform & asset
  -- but allows versioning different curves
  -- also im just shit at sql and dont want to figure out multi-key constraints 
  rate_name VARCHAR NOT NULL UNIQUE,
  -- used by: solend, tulip
  min_borrow_rate FLOAT8 NOT NULL DEFAULT 0,
  -- used by: solend, tulip
  max_borrow_rate FLOAT8 NOT NULL DEFAULT 0,
  -- used by: solend, tulip
  optimal_borrow_rate FLOAT8 NOT NULL DEFAULT 0,
  -- used by: solend, tulip
  optimal_utilization_rate FLOAT8 NOT NULL DEFAULT 0,
  -- used by: tulip
  degen_borrow_rate FLOAT8 NOT NULL DEFAULT 0,
  -- used by: tulip
  degen_utilization_rate FLOAT8 NOT NULL DEFAULT 0
);