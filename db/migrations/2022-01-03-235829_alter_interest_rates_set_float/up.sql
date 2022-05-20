-- Your SQL goes here
ALTER TABLE interest_rate ALTER COLUMN available_amount TYPE float8 USING available_amount::float8;
ALTER TABLE interest_rate ALTER COLUMN borrowed_amount TYPE float8 USING borrowed_amount::float8;