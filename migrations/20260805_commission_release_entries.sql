-- Multi-entry commission release ledger: an admin can record more than one dated
-- release amount against a subject + biweekly period (e.g. a partial release now,
-- another later); the frontend sums these to compute Remaining, matching the
-- "PARTIAL PAYMENT" list in the source spreadsheet.

CREATE TABLE IF NOT EXISTS public.commission_release_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    subject_agent_id TEXT NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    amount DOUBLE PRECISION NOT NULL CHECK (amount > 0),
    paid_at DATE NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS commission_release_entries_project_period_idx
    ON public.commission_release_entries (project_id, subject_agent_id, period_start);
