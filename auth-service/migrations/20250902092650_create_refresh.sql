CREATE TABLE
    refresh_tokens (
        jti uuid PRIMARY KEY,
        user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        session_id uuid NOT NULL REFERENCES sessions (id) ON DELETE CASCADE,
        token_hash bytea NOT NULL,
        parent_jti uuid NULL REFERENCES refresh_tokens (jti) ON DELETE SET NULL,
        expires_at timestamptz NOT NULL,
        rotated_at timestamptz NULL,
        revoked_at timestamptz NULL,
        CONSTRAINT chk_refresh_token_hash_len CHECK (octet_length(token_hash) = 32) -- SHA-256
    );

CREATE UNIQUE INDEX idx_refresh_by_hash ON refresh_tokens (token_hash);

CREATE INDEX idx_refresh_by_family ON refresh_tokens (family_id);

CREATE INDEX idx_refresh_by_user ON refresh_tokens (user_id);

CREATE INDEX idx_refresh_by_sess ON refresh_tokens (session_id);

CREATE INDEX idx_refresh_by_exp ON refresh_tokens (expires_at);

CREATE INDEX idx_refresh_active ON refresh_tokens (family_id, expires_at)
WHERE
    rotated_at IS NULL
    AND revoked_at IS NULL;