ALTER TABLE token_price ADD COLUMN coin_in_lp FLOAT NOT NULL DEFAULT 0;
ALTER TABLE token_price ADD COLUMN pc_in_lp FLOAT NOT NULL DEFAULT 0;