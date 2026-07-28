-- Pricelist entries ("lots") for a project. Contract price is always
-- derived server-side from area * rate to keep it consistent.
CREATE TABLE IF NOT EXISTS public.lots (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id     UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    block          VARCHAR(20) NOT NULL,
    lot            VARCHAR(20) NOT NULL,
    lot_type       VARCHAR(30) NOT NULL DEFAULT 'Inner'
                       CHECK (lot_type IN ('Inner', 'Commercial', 'Corner', 'Commercial / Corner')),
    area           DOUBLE PRECISION NOT NULL,
    rate           DOUBLE PRECISION NOT NULL,
    contract_price DOUBLE PRECISION NOT NULL,
    owner_buyer    VARCHAR(255),
    on_hold        BOOLEAN NOT NULL DEFAULT false,
    status         VARCHAR(20) NOT NULL DEFAULT 'Available'
                       CHECK (status IN ('Available', 'Hold', 'Reserved', 'Sold')),
    created_at     BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    updated_at     BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    UNIQUE (project_id, block, lot)
);

CREATE INDEX IF NOT EXISTS idx_lots_project_id ON public.lots(project_id);
