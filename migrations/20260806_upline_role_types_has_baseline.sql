-- Whether an upline role earns its base % on every sale project-wide (a "baseline"
-- cut, even outside their own team) or only from their own tree's sales. Defaults to
-- true so existing roles (Lead Broker, Titling Officer) keep their current behavior
-- unchanged; new "normal" upline roles can opt out of the baseline cut.

ALTER TABLE public.upline_role_types
    ADD COLUMN IF NOT EXISTS has_baseline BOOLEAN NOT NULL DEFAULT true;
