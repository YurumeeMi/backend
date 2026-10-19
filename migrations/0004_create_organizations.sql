CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT organizations_name_not_empty
    CHECK (length(trim(name)) > 0 )
);

 CREATE TABLE memberships (
    organization_id UUID NOT NULL 
    REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL
    REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL
    CHECK(role IN ('owner', 'admin', 'member')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (organization_id, user_id)
 );
CREATE INDEX idx_memberships_user_id
       ON memberships(user_id)