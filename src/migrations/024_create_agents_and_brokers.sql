-- Licensed real-estate brokers. Each broker is a distinct entity from a
-- selling agent (own PRC license, own default commission rate) and may
-- supervise zero or more agents.
CREATE TABLE IF NOT EXISTS public.brokers (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id         SMALLINT NOT NULL DEFAULT 1 REFERENCES public.company_settings(id),
    name               VARCHAR(255) NOT NULL,
    broker_code        VARCHAR(100) NOT NULL,
    prc_license_number VARCHAR(100) NOT NULL,
    contact_number     VARCHAR(50) NOT NULL DEFAULT '',
    email              VARCHAR(255) NOT NULL DEFAULT '',
    commission_rate    DOUBLE PRECISION NOT NULL DEFAULT 0,
    status             VARCHAR(20) NOT NULL DEFAULT 'Active'
                           CHECK (status IN ('Active', 'Inactive')),
    created_at         BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    updated_at         BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    CONSTRAINT brokers_broker_code_unique UNIQUE (company_id, broker_code)
);

CREATE INDEX IF NOT EXISTS idx_brokers_company_id ON public.brokers(company_id);

-- Selling agents. Reporting to a broker is optional (some agents work
-- directly under the company with no broker on file), so broker_id is
-- nullable and cleared (not cascaded) if the broker record is removed.
CREATE TABLE IF NOT EXISTS public.selling_agents (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id         SMALLINT NOT NULL DEFAULT 1 REFERENCES public.company_settings(id),
    broker_id          UUID REFERENCES public.brokers(id) ON DELETE SET NULL,
    name               VARCHAR(255) NOT NULL,
    agent_code         VARCHAR(100) NOT NULL,
    prc_license_number VARCHAR(100),
    contact_number     VARCHAR(50) NOT NULL DEFAULT '',
    email              VARCHAR(255) NOT NULL DEFAULT '',
    commission_rate    DOUBLE PRECISION NOT NULL DEFAULT 0,
    status             VARCHAR(20) NOT NULL DEFAULT 'Active'
                           CHECK (status IN ('Active', 'Inactive')),
    created_at         BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    updated_at         BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    CONSTRAINT selling_agents_agent_code_unique UNIQUE (company_id, agent_code)
);

CREATE INDEX IF NOT EXISTS idx_selling_agents_company_id ON public.selling_agents(company_id);
CREATE INDEX IF NOT EXISTS idx_selling_agents_broker_id ON public.selling_agents(broker_id);

-- Give contracts a real link to these tables without disturbing the
-- existing free-text snapshot fields (marketing_representative, agent_code,
-- selling_agent_id), which stay untouched as-is.
ALTER TABLE public.contracts
    ADD COLUMN IF NOT EXISTS agent_id  UUID REFERENCES public.selling_agents(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS broker_id UUID REFERENCES public.brokers(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_contracts_agent_id ON public.contracts(agent_id);
CREATE INDEX IF NOT EXISTS idx_contracts_broker_id ON public.contracts(broker_id);

-- Default commission percentage per role. A fixed, admin-editable lookup
-- table (rows are seeded once below and never added to/removed via the API)
-- rather than a free-for-all table, since the set of roles is closed.
CREATE TABLE IF NOT EXISTS public.commission_rates (
    role            VARCHAR(50) PRIMARY KEY,
    commission_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at      BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT)
);

INSERT INTO public.commission_rates (role, commission_rate)
VALUES
    ('Lead Broker', 0),
    ('Titling Officer', 0),
    ('Agent', 0)
ON CONFLICT (role) DO NOTHING;
