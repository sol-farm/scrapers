ALTER TABLE v1_user_farm ADD COLUMN leveraged_farm VARCHAR NOT NULL DEFAULT '';
ALTER TABLE v1_liquidated_position ADD COLUMN leveraged_farm VARCHAR NOT NULL DEFAULT '';
