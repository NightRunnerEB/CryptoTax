CREATE TABLE
    users (
        id uuid PRIMARY KEY,
        email text NOT NULL UNIQUE,
        password_hash text NOT NULL,
        status user_status NOT NULL DEFAULT 'Pending',
        created_at timestamptz NOT NULL DEFAULT now ()
    );

CREATE UNIQUE INDEX idx_users_email_lower ON users ((lower(email)));