CREATE TABLE IF NOT EXISTS public.projects (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id SMALLINT NOT NULL DEFAULT 1 REFERENCES public.company_settings(id),
    name       VARCHAR(255) NOT NULL,
    created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT)
);

CREATE INDEX IF NOT EXISTS idx_projects_company_id ON public.projects(company_id);
