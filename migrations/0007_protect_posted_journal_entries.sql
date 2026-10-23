CREATE OR REPLACE FUNCTION prevent_posted_journal_entries_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'posted' THEN
        RAISE EXCEPTION
            'posted journal entries cannot be modified';
    END IF;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;

    RETURN NEW;
END;

$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_posted_journal_entries
BEFORE UPDATE OR DELETE ON journal_entries
FOR EACH ROW
EXECUTE FUNCTION  prevent_posted_journal_entries_changes();
