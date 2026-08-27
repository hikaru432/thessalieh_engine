-- upline_role_types introduced configurable root roles (e.g. Normal Upline), but
-- roster still had CHECK (role IN ('Lead Broker', 'Titling Officer', 'Agent')) from
-- the original table definition. Validate allowed roles in the API instead — same
-- pattern as users.role after 20260727_upline_role_types.sql.

ALTER TABLE public.roster DROP CONSTRAINT IF EXISTS roster_role_check;

ALTER TABLE public.roster
    ALTER COLUMN role TYPE VARCHAR(100);
