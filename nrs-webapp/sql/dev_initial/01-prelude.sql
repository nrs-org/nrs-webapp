CREATE COLLATION case_insensitive (
    provider = icu,
    locale = 'und-u-ks-level2',
    deterministic = false
);

-- BEGIN SQL BLOCK
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';
-- END SQL BLOCK
