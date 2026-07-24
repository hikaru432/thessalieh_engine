-- Per buyer/agent row indicators on a commission biweek (e.g. Half amort).

CREATE TABLE IF NOT EXISTS public.commission_row_meta (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    subject_agent_id TEXT NOT NULL,
    row_key TEXT NOT NULL,
    period_start DATE NOT NULL,
    other_flag TEXT NOT NULL DEFAULT 'none'
        CHECK (other_flag IN ('none', 'half', 'full')),
    updated_at BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT commission_row_meta_unique
        UNIQUE (project_id, subject_agent_id, row_key, period_start)
);

CREATE INDEX IF NOT EXISTS commission_row_meta_project_idx
    ON public.commission_row_meta (project_id, subject_agent_id, period_start);
