CREATE TABLE
    imports (
        id uuid PRIMARY KEY,
        user_id uuid NOT NULL,
        source text NOT NULL,
        file_name text,
        status text NOT NULL,
        total_count integer NOT NULL DEFAULT 0,
        error_summary text,
        error_details jsonb,
        created_at timestamptz NOT NULL DEFAULT now (),
        completed_at timestamptz
    );

ALTER TABLE imports ADD CONSTRAINT chk_imports_status CHECK (
    status IN ('processing', 'completed', 'failed', 'rolledBack')
);

CREATE INDEX idx_imports_user_created ON imports (user_id, created_at DESC);

CREATE INDEX idx_imports_user_source_created ON imports (user_id, source, created_at DESC);
