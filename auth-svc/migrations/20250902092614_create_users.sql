CREATE TABLE 
    IF NOT EXISTS users (
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
        email citext NOT NULL UNIQUE,
        password_hash text NOT NULL,
        status user_status NOT NULL DEFAULT 'Pending',
        created_at timestamptz NOT NULL DEFAULT now()
    );