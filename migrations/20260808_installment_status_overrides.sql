-- Manual admin override of the Payment Tracker's per-month status pill,
-- takes priority over the auto-computed status derived from Cash Flow payments.

CREATE TABLE IF NOT EXISTS public.installment_status_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    contract_id UUID NOT NULL REFERENCES public.contracts(id) ON DELETE CASCADE,
    year INT NOT NULL,
    month INT NOT NULL CHECK (month BETWEEN 0 AND 11),
    status TEXT NOT NULL DEFAULT '' CHECK (status IN ('', 'Paid', 'Half', 'Hold')),
    updated_at BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT installment_status_overrides_unique
        UNIQUE (contract_id, year, month)
);

CREATE INDEX IF NOT EXISTS installment_status_overrides_project_idx
    ON public.installment_status_overrides (project_id, year);
