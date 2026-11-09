
ALTER TABLE accounts
ADD COLUMN opening_balance NUMERIC(20, 4) NOT NULL DEFAULT 0
CHECK (opening_balance >= 0);
