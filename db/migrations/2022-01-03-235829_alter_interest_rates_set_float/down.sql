ALTER TABLE interest_rate ALTER COLUMN available_amount TYPE bigint USING available_amount::bigint;
ALTER TABLE interest_rate ALTER COLUMN borrowed_amount TYPE bigint USING borrowed_amount::bigint;