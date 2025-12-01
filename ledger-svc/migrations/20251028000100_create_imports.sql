CREATE TABLE
    imports (
        id uuid PRIMARY KEY,
        tenant_id uuid NOT NULL,
        wallet text NOT NULL,
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
    status IN (
        'processing',
        'completed',
        'failed',
        'rolledBack'
    )
);

CREATE INDEX idx_imports_tenant_created ON imports (tenant_id, created_at DESC);

CREATE INDEX idx_imports_tenant_wallet_created ON imports (tenant_id, wallet, created_at DESC);