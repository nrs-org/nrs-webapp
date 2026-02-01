CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS app_user (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  username TEXT COLLATE case_insensitive NOT NULL,
  email TEXT COLLATE case_insensitive NOT NULL,
  email_verified_at TIMESTAMPTZ,
  password_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(username),
  UNIQUE(email)
);

CREATE INDEX app_user_verified_idx
ON app_user (email_verified_at)
WHERE email_verified_at IS NOT NULL;

CREATE TRIGGER update_app_user_updated_at
    BEFORE UPDATE ON app_user
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
