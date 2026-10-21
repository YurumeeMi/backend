CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL 
    REFERENCES organizations(id) ON DELETE RESTRICT,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL 
    CHECK (account_type IN (
        'asset',
        'liability',
        'equity',
        'revenue',
        'expense'
    )),
    parent_account_id UUID,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT accounts_name_not_empty
    CHECK (length(trim(name)) > 0),

    CONSTRAINT accounts_code_not_empty
    CHECK (length(trim(code)) > 0),

    CONSTRAINT account_org_id_id_unique
    UNIQUE (organization_id, id),

    CONSTRAINT accounts_org_code_unique
    UNIQUE (organization_id, code),

    CONSTRAINT accounts_parent_same_org
    FOREIGN KEY (organization_id, parent_account_id)
    REFERENCES accounts(organization_id, id)

);

CREATE TABLE journal_entries (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL
    REFERENCES organizations(id) ON DELETE RESTRICT,
    entry_date DATE NOT NULL,
    description TEXT NOT NULL,
    reference TEXT,
    status TEXT NOT NULL DEFAULT 'draft'
    CHECK (status IN ('draft', 'posted')),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    posted_at TIMESTAMPTZ,

    CONSTRAINT journal_entries_description_not_empty
    CHECK (length(trim(description)) > 0),

    CONSTRAINT journal_entries_org_id_id_unique
    UNIQUE (organization_id, id),

    CONSTRAINT journal_entries_posted_timestamp
    CHECK (
        (status = 'draft' AND posted_at IS NULL)
        OR
        (status = 'posted' AND posted_at IS NOT NULL)
    )
);

CREATE INDEX idx_journal_entries_org_date
ON journal_entries(organization_id, entry_date DESC);

CREATE TABLE journal_lines (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL,
    journal_entry_id UUID NOT NULL,
    account_id UUID NOT NULL,
    description TEXT,
    debit NUMERIC(20, 4) NOT NULL DEFAULT 0,
    credit NUMERIC(20, 4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT journal_lines_entry_same_org
    FOREIGN KEY (organization_id, journal_entry_id)
    REFERENCES journal_entries(organization_id, id)
    ON DELETE RESTRICT,

    CONSTRAINT journal_lines_account_same_org
    FOREIGN KEY (organization_id, account_id)
    REFERENCES accounts(organization_id, id)
    ON DELETE RESTRICT,

    CONSTRAINT journal_lines_one_sided_amount
    CHECK (
    (debit > 0 AND credit = 0)
    OR
    (credit > 0 AND debit = 0)
    )
);

CREATE INDEX idx_journal_lines_entry
    ON journal_lines(organization_id, journal_entry_id);

CREATE INDEX idx_journal_lines_account
    ON journal_lines(organization_id, account_id);
