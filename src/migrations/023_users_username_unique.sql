-- Login and registration now identify accounts by username, so it must be unique.
-- NOTE: if this fails, resolve duplicates first, e.g.:
--   SELECT username, count(*) FROM public.users GROUP BY username HAVING count(*) > 1;
ALTER TABLE public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);
