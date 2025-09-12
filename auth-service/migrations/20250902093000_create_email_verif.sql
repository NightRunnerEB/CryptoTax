CREATE TABLE
    IF NOT EXISTS email_verifications (
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        token_hash bytea NOT NULL,
        sent_to citext NOT NULL,
        expires_at timestamptz NOT NULL,
        consumed_at timestamptz NULL,
        created_at timestamptz NOT NULL DEFAULT now(),
        CONSTRAINT chk_email_verif_hash_len CHECK (octet_length(token_hash) = 32)
    );

CREATE UNIQUE INDEX IF NOT EXISTS idx_email_verif_by_hash ON email_verifications (token_hash);

CREATE INDEX IF NOT EXISTS idx_email_verif_by_user ON email_verifications (user_id);

CREATE INDEX IF NOT EXISTS idx_email_verif_active_by_user ON email_verifications (user_id, expires_at)
WHERE
    consumed_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_email_verif_by_exp ON email_verifications (expires_at);