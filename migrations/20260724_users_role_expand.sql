-- Expand users.role to allow roster-synced roles, then backfill from roster.

DO $$
DECLARE
  constraint_row record;
BEGIN
  FOR constraint_row IN
    SELECT conname
    FROM pg_constraint
    WHERE conrelid = 'public.users'::regclass
      AND contype = 'c'
      AND pg_get_constraintdef(oid) ILIKE '%role%'
  LOOP
    EXECUTE format(
      'ALTER TABLE public.users DROP CONSTRAINT IF EXISTS %I',
      constraint_row.conname
    );
  END LOOP;
END $$;

ALTER TABLE public.users
  ADD CONSTRAINT users_role_check
  CHECK (role IN ('User', 'Admin', 'Lead Broker', 'Titling Officer'));

UPDATE public.users u
SET role = r.role,
    updated_at = extract(epoch FROM now())::bigint
FROM public.roster r
WHERE r.user_id = u.id
  AND r.role IN ('Lead Broker', 'Titling Officer')
  AND u.role NOT IN ('Admin');
