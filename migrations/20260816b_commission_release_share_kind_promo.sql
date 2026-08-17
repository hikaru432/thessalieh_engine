-- Constrain commission release share_kind to the known ledgers used by the
-- admin UI: Baseline (base), Direct buyer (pool), Promo (promo), or null for a
-- plain agent's combined ledger. Promo releases must not share the agent-null
-- bucket or they'd merge into the As-agent tab.
--
-- Cash-out aggregation (commission_release_summary) already SUMs every entry
-- regardless of share_kind — no change needed there.

ALTER TABLE public.commission_release_entries
    DROP CONSTRAINT IF EXISTS commission_release_entries_share_kind_check;

ALTER TABLE public.commission_release_entries
    ADD CONSTRAINT commission_release_entries_share_kind_check
    CHECK (share_kind IS NULL OR share_kind IN ('base', 'pool', 'promo'));

ALTER TABLE public.commission_release_credits
    DROP CONSTRAINT IF EXISTS commission_release_credits_share_kind_check;

ALTER TABLE public.commission_release_credits
    ADD CONSTRAINT commission_release_credits_share_kind_check
    CHECK (share_kind IS NULL OR share_kind IN ('base', 'pool', 'promo'));
