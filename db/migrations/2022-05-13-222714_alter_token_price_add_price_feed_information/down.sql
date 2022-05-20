ALTER TABLE token_price DROP COLUMN period_start; 
ALTER TABLE token_price DROP COLUMN period_end; 
ALTER TABLE token_price DROP COLUMN period_observed_prices;
ALTER TABLE token_price DROP COLUMN period_running_average;
ALTER TABLE token_price DROP COLUMN last_period_average;
ALTER TABLE token_price DROP COLUMN feed_stopped;
ALTER TABLE token_price DROP COLUMN token_mint;