CREATE TABLE
    outbox (
        id serial PRIMARY KEY,
        event_id uuid NOT NULL,
        tenant_id uuid NOT NULL,
        aggregate_type text NOT NULL,
        aggregate_id uuid NOT NULL,
        event_type text NOT NULL,
        event_version integer NOT NULL,
        payload jsonb NOT NULL,
        headers jsonb,
        status text NOT NULL DEFAULT 'pending',
        attempts integer NOT NULL DEFAULT 0,
        last_error text,
        created_at timestamptz NOT NULL DEFAULT now (),
        published_at timestamptz
    );

ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'processing', 'published', 'failed'));

CREATE UNIQUE INDEX idx_outbox_event_id ON outbox (event_id);

CREATE INDEX idx_outbox_status_created ON outbox (status, created_at)
WHERE
    status = 'pending';

CREATE INDEX idx_outbox_aggregate ON outbox (aggregate_type, aggregate_id);