CREATE TYPE USER_ONE_TIME_TOKEN_PURPOSE AS ENUM (
  'EMAIL_VERIFICATION',
  'PASSWORD_RESET'
);

CREATE TABLE IF NOT EXISTS user_one_time_token (
  user_id TEXT REFERENCES app_user(id) ON DELETE CASCADE,
  token_hash TEXT NOT NULL,
  purpose USER_ONE_TIME_TOKEN_PURPOSE NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_used_at TIMESTAMPTZ,
  request_ip TEXT,
  user_agent TEXT
);

CREATE UNIQUE INDEX user_token_one_active_per_purpose
ON user_one_time_token (user_id, purpose)
WHERE last_used_at IS NULL;

CREATE UNIQUE INDEX user_token_hash_unique
ON user_one_time_token (token_hash);

CREATE INDEX user_token_expires_idx
ON user_one_time_token (expires_at);
