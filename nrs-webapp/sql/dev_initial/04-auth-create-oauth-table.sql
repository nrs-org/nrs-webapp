CREATE TABLE app_user_oauth_link(
  user_id UUID REFERENCES app_user(id) ON DELETE CASCADE,
  provider TEXT NOT NULL,
  provider_user_id TEXT,
  issuer TEXT,
  access_token BYTEA,
  refresh_token BYTEA,
  access_token_expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  revoked_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX app_user_oauth_link_user_provider_unique
ON app_user_oauth_link (user_id, provider)
WHERE revoked_at IS NULL;

CREATE UNIQUE INDEX app_user_oauth_link_provider_user_unique
ON app_user_oauth_link (provider, COALESCE(provider_user_id, '__NULL__'))
WHERE revoked_at IS NULL;

CREATE TRIGGER update_app_user_oauth_link_updated_at
    BEFORE UPDATE ON app_user_oauth_link
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
