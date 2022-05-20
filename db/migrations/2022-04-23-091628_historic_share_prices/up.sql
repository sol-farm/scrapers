-- the ltv of v1 lending obligations
CREATE TABLE historic_tshare_price (
  id BIGSERIAL PRIMARY KEY,
  farm_name VARCHAR NOT NULL,
  price FLOAT8 NOT NULL DEFAULT 0,
  total_supply FLOAT8 NOT NULL DEFAULT 0,
  -- currently unused, but will be used later
  holder_count FLOAT8 NOT NULL DEFAULT 0,
  scraped_at TIMESTAMPTZ NOT NULL
);