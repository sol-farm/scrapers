-- maintains a moving average of lending interest rates
-- that is, the interest rates for lending/supplying assets to a platform
CREATE TABLE interest_rate_moving_average (
    id BIGSERIAL PRIMARY KEY,
    platform VARCHAR NOT NULL,
    asset VARCHAR NOT NULL,
    -- combination of PLATFORM-ASSET (MANGO-RAY, MANGO-USDC, etc..)
    rate_name VARCHAR NOT NULL UNIQUE,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    -- the current moving average for lending interest rates in this period
    period_running_average FLOAT8 NOT NULL,
    period_observed_rates FLOAT8[] NOT NULL DEFAULT '{}',
    last_period_running_average FLOAT8 NOT NULL
);