CREATE TABLE
    IF NOT EXISTS sessions (
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        status session_status NOT NULL DEFAULT 'Active',
        created_at timestamptz NOT NULL DEFAULT now (),
        last_seen_at timestamptz NOT NULL DEFAULT now (),
        ip text,
        user_agent text
    );

CREATE INDEX idx_sessions_user_id ON sessions (user_id);