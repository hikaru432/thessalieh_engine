-- Per-contract history of agent_commission_split_months changes, so a mid-stream
-- split change only affects months from its effective date forward. A contract that
-- has never had its split changed has zero rows here (lazy genesis — see
-- change_contract_split in contract_split_history.rs).

CREATE TABLE IF NOT EXISTS public.contract_split_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id UUID NOT NULL REFERENCES public.contracts(id) ON DELETE CASCADE,
    split_months INTEGER NOT NULL CHECK (split_months BETWEEN 1 AND 120),
    effective_month DATE NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT contract_split_history_unique UNIQUE (contract_id, effective_month)
);

CREATE INDEX IF NOT EXISTS contract_split_history_contract_idx
    ON public.contract_split_history (contract_id, effective_month);
