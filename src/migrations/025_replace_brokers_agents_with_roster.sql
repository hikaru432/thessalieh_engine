-- Replace the separate brokers/selling_agents tables with a single roster
-- table now that Titling Officer joins the roster and every entry must map
-- to a real user account instead of a free-typed name/contact/email (those
-- live on public.users and are looked up by user_id).
DROP TABLE IF EXISTS public.selling_agents CASCADE;
DROP TABLE IF EXISTS public.brokers CASCADE;

ALTER TABLE public.contracts DROP COLUMN IF EXISTS agent_id;
ALTER TABLE public.contracts DROP COLUMN IF EXISTS broker_id;

CREATE TABLE IF NOT EXISTS public.roster (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id         SMALLINT NOT NULL DEFAULT 1 REFERENCES public.company_settings(id),
    user_id            UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    role               VARCHAR(20) NOT NULL
                           CHECK (role IN ('Lead Broker', 'Titling Officer', 'Agent')),
    -- Who this member reports to (typically a Lead Broker or Titling
    -- Officer). Null for roots. Only meaningful when role = 'Agent'
    -- (enforced in the API, not here). Cleared, not cascaded, if the
    -- supervisor's own roster entry is removed.
    broker_id          UUID REFERENCES public.roster(id) ON DELETE SET NULL,
    code               VARCHAR(100) NOT NULL,
    prc_license_number VARCHAR(100),
    commission_rate    DOUBLE PRECISION NOT NULL DEFAULT 0,
    status             VARCHAR(20) NOT NULL DEFAULT 'Active'
                           CHECK (status IN ('Active', 'Inactive')),
    created_at         BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    updated_at         BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    -- One roster role per user account.
    CONSTRAINT roster_user_id_unique UNIQUE (user_id),
    CONSTRAINT roster_code_unique UNIQUE (company_id, code)
);

CREATE INDEX IF NOT EXISTS idx_roster_company_id ON public.roster(company_id);
CREATE INDEX IF NOT EXISTS idx_roster_broker_id ON public.roster(broker_id);

ALTER TABLE public.contracts
    ADD COLUMN IF NOT EXISTS agent_id  UUID REFERENCES public.roster(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS broker_id UUID REFERENCES public.roster(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_contracts_agent_id ON public.contracts(agent_id);
CREATE INDEX IF NOT EXISTS idx_contracts_broker_id ON public.contracts(broker_id);
