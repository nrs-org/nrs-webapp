CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS app_user (
                id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
                username TEXT NOT NULL,
                email TEXT NOT NULL,
                email_verified_at TIMESTAMPTZ,
                password_hash TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(email),
                UNIQUE(id)
            );

CREATE TABLE IF NOT EXISTS session (
                id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
                user_id TEXT NOT NULL,
                user_agent TEXT,
                ip_address TEXT,
                token TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY (user_id) REFERENCES app_user(id) ON DELETE CASCADE
            );

CREATE TABLE IF NOT EXISTS password_reset_token (
                id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
                user_id TEXT NOT NULL,
                token TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY(user_id) REFERENCES app_user(id) ON DELETE CASCADE,
                UNIQUE(token)
            );

CREATE TABLE IF NOT EXISTS oauth_account (
                id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
                user_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                subject TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                FOREIGN KEY(user_id) REFERENCES app_user(id) ON DELETE CASCADE,
                UNIQUE(user_id, provider, subject)
            );

CREATE TABLE IF NOT EXISTS oauth_challenge (
                id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
                challenge_id TEXT NOT NULL,
                challenge TEXT NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(challenge_id)
            );

CREATE INDEX IF NOT EXISTS idx_users_email ON app_user(email);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON session(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON session(expires_at);
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_user_id ON oauth_account(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_provider_subject ON oauth_account(provider, subject);
CREATE INDEX IF NOT EXISTS idx_passkey_challenges_expires_at ON oauth_challenge(expires_at);
