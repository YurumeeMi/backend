CREATE TABLE sessions (
    id UUID NOT NULL
    REFERENCES users(user_id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id 
ON sessions(user_id);
CREATE INDEX
idx_sessions_expires_at ON sessions(expires_at);