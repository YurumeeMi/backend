CREATE OR REPLACE FUNCTION prevent_posted_journal_changes()
RETURNS TRIGGER AS $$
DECLARE 
      entry_status TEXT;
      target_entry_id UUID;
      target_org_id UUID;
BEGIN
    IF TG_OP = 'DELETE' THEN
    target_entry_id := OLD.journal_entry_id;
    target_org_id := OLD.organization_id;
    ELSE
    target_entry_id := NEW.journal_entry_id;
    target_org_id := NEW.organization_id;
    END IF;
    SELECT status
    INTO  entry_status
    FROM journal_entries
    WHERE id = target_entry_id
          AND organization_id = target_org_id
    FOR UPDATE;

    IF entry_status = 'posted' THEN
       RAISE EXCEPTION
       'Journal cannot be modified of a posted entry';
    END IF;

    IF TG_OP = 'UPDATE'
    AND (
       OLD.journal_entry_id IS DISTINCT FROM NEW.journal_entry_id
       OR OLD.organization_id IS DISTINCT FROM NEW.organization_id
    ) THEN
      RAISE EXCEPTION 
      'Cannot move a journal line to another entry or org';
    END IF;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_posted_journal_changes
BEFORE INSERT OR UPDATE OR DELETE ON journal_lines
FOR EACH ROW
EXECUTE FUNCTION prevent_posted_journal_changes();