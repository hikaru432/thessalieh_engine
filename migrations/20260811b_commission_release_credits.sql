-- A commission release can now exceed everything currently owed (previously
-- blocked client-side). The excess is recorded here as a standalone credit
-- grant rather than attached to any period — periods aren't stored server-side
-- at all (see commission_release_entries), and the excess genuinely has nowhere
-- owed to land yet. buildCommissionWaterfall (frontend) drains this pool, oldest
-- grant first, into whichever period next has a real unpaid balance — including
-- one that doesn't exist yet at the time the credit is granted — the same way it
-- already drains recorded release entries, so nothing new needs to happen the
-- moment a future period's own commission comes due.
CREATE TABLE IF NOT EXISTS public.commission_release_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    subject_agent_id TEXT NOT NULL,
    -- Scopes this credit to a root's Baseline vs Direct buyer share, same as
    -- commission_release_entries.share_kind. Null for a plain agent, whose
    -- release has no split.
    share_kind TEXT,
    amount DOUBLE PRECISION NOT NULL CHECK (amount > 0),
    -- The date of the release that generated this credit — not a due date,
    -- just when the overage happened, for display/ordering.
    paid_at DATE NOT NULL,
    note TEXT,
    created_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS commission_release_credits_project_subject_idx
    ON public.commission_release_credits (project_id, subject_agent_id);
