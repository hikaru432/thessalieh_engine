ALTER TABLE public.users DROP CONSTRAINT IF EXISTS users_role_check;

ALTER TABLE public.users
  ADD CONSTRAINT users_role_check
  CHECK (role IN ('User', 'Admin', 'Lead Broker', 'Titling Officer', 'Agent'));

UPDATE public.users u
SET role = r.role,
    updated_at = extract(epoch FROM now())::bigint
FROM public.roster r
WHERE r.user_id = u.id
  AND u.role NOT IN ('Admin');
