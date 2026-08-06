-- Scopes a release entry to a specific commission component ("base" = baseline,
-- "pool" = own-tree override) so a root's Baseline and Direct buyer sections can be
-- released independently (e.g. paying out Direct buyer first while Baseline stays
-- unpaid). NULL for plain-agent release entries, which have no such split.

ALTER TABLE public.commission_release_entries
    ADD COLUMN IF NOT EXISTS share_kind TEXT NULL;
