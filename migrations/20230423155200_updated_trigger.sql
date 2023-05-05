-- Add migration script here
CREATE OR REPLACE FUNCTION trigger_set_updated_at_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE PROCEDURE create_updated_at_trigger(t name)
AS $$
BEGIN
    execute format('CREATE OR REPLACE TRIGGER %s_trigger_set_updated_at_timestamp
BEFORE UPDATE ON %s FOR EACH ROW
EXECUTE PROCEDURE trigger_set_updated_at_timestamp();', t, t);
END;
$$ LANGUAGE plpgsql;