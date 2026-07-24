-- Commission release status per agent (or LB/TO subject) × biweekly period.

CREATE TABLE IF NOT EXISTS public.commission_period_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    subject_agent_id TEXT NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    status TEXT NOT NULL DEFAULT 'not_yet'
        CHECK (status IN ('not_yet', 'partial', 'pending', 'paid')),
    partial_amount DOUBLE PRECISION,
    partial_paid_at DATE,
    updated_at BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT commission_period_status_unique
        UNIQUE (project_id, subject_agent_id, period_start)
);

CREATE INDEX IF NOT EXISTS commission_period_status_project_period_idx
    ON public.commission_period_status (project_id, period_start, period_end);
