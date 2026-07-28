-- Buyer contracts. Lot fields are a denormalized snapshot taken at contract
-- time (a contract's price/terms shouldn't retroactively change if the
-- pricelist entry is edited later) — lot_id is an optional live link back to
-- the catalog entry, used to keep the lot's owner/status in sync.
-- Status ("Fully Paid" / "Needs Attention" / "On Track") is derived from
-- balance + next_due_date at query time, not stored.
CREATE TABLE IF NOT EXISTS public.contracts (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id               UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    lot_id                   UUID REFERENCES public.lots(id) ON DELETE SET NULL,

    buyer_name               VARCHAR(255) NOT NULL,
    buyer_address            TEXT NOT NULL DEFAULT '',
    buyer_gmail              VARCHAR(255) NOT NULL DEFAULT '',
    buyer_contact            VARCHAR(50) NOT NULL DEFAULT '',

    lot_block                VARCHAR(20) NOT NULL,
    lot_lot                  VARCHAR(20) NOT NULL,
    lot_area                 DOUBLE PRECISION NOT NULL DEFAULT 0,
    lot_type                 VARCHAR(30) NOT NULL DEFAULT 'Inner',
    lot_rate                 DOUBLE PRECISION NOT NULL DEFAULT 0,

    contract_price           DOUBLE PRECISION NOT NULL,
    payment_plan             VARCHAR(20) NOT NULL DEFAULT 'installment'
                                  CHECK (payment_plan IN ('installment', 'half', 'full')),
    initial_payment          DOUBLE PRECISION NOT NULL DEFAULT 0,
    term_years               INT NOT NULL DEFAULT 0,
    monthly_amortization     DOUBLE PRECISION NOT NULL DEFAULT 0,
    due_day                  INT NOT NULL DEFAULT 15,
    next_due_date            DATE NOT NULL,
    approval_at              DATE,

    -- Sales agent / broker — the agent hierarchy is local-only (Marketing
    -- feature), so this is a plain reference with no foreign key.
    marketing_representative VARCHAR(255) NOT NULL DEFAULT '',
    agent_code               VARCHAR(100) NOT NULL DEFAULT '',
    selling_agent_id         VARCHAR(255),
    source_of_buyer          TEXT[] NOT NULL DEFAULT '{}',
    other_source             VARCHAR(255) NOT NULL DEFAULT '',

    particulars              VARCHAR(255) NOT NULL DEFAULT '',
    created_at               BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT),
    updated_at               BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT)
);

CREATE INDEX IF NOT EXISTS idx_contracts_project_id ON public.contracts(project_id);
CREATE INDEX IF NOT EXISTS idx_contracts_lot_id ON public.contracts(lot_id);

-- Payment ledger. Total paid / balance / progress are always derived by
-- summing this table for a contract — never stored as a mutable running
-- total, so they can't drift out of sync.
CREATE TABLE IF NOT EXISTS public.payments (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id    UUID NOT NULL REFERENCES public.contracts(id) ON DELETE CASCADE,
    amount         DOUBLE PRECISION NOT NULL,
    method         VARCHAR(20) NOT NULL DEFAULT 'cash'
                       CHECK (method IN ('cash', 'card', 'gcash', 'maya')),
    months_covered INT NOT NULL DEFAULT 1,
    paid_at        DATE NOT NULL,
    created_at     BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::BIGINT)
);

CREATE INDEX IF NOT EXISTS idx_payments_contract_id ON public.payments(contract_id);

-- ---------------------------------------------------------------------------
-- Seed: Villamor Village becomes a real project so its lots and buyer
-- contracts live in the same tables as every other project, instead of a
-- frontend-only mock. Safe to re-run (all inserts are ON CONFLICT DO NOTHING
-- or keyed off a fixed UUID).
-- ---------------------------------------------------------------------------
INSERT INTO public.projects (id, company_id, name)
VALUES ('11111111-1111-4111-8111-111111111111', 1, 'Villamor Village')
ON CONFLICT (id) DO NOTHING;

INSERT INTO public.lots (project_id, block, lot, lot_type, area, rate, contract_price, status)
VALUES
    ('11111111-1111-4111-8111-111111111111', '1', '1', 'Commercial / Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '2', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '3', 'Commercial / Corner', 115, 4500, 517500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '4', 'Commercial', 112, 4500, 504000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '5', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '6', 'Commercial', 113, 4500, 508500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '7', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '8', 'Commercial', 114, 4500, 513000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '9', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '10', 'Commercial', 115, 4500, 517500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '11', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '12', 'Commercial', 116, 4500, 522000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '13', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '14', 'Commercial', 117, 4500, 526500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '15', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '16', 'Commercial / Corner', 157, 4500, 706500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '1', '17', 'Commercial / Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '1', 'Commercial / Corner', 112, 4500, 504000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '2', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '3', 'Commercial / Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '4', 'Commercial', 107, 4500, 481500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '5', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '6', 'Commercial', 106, 4500, 477000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '7', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '8', 'Commercial', 105, 4500, 472500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '9', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '10', 'Commercial', 104, 4500, 468000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '11', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '12', 'Commercial', 103, 4500, 463500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '13', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '14', 'Commercial', 102, 4500, 459000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '15', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '16', 'Commercial / Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '2', '17', 'Corner', 143, 4500, 643500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '1', 'Corner', 121, 4500, 544500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '2', 'Commercial / Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '3', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '4', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '5', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '6', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '7', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '8', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '9', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '10', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '11', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '12', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '13', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '14', 'Corner', 132, 4500, 594000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '3', '15', 'Corner', 110, 4500, 495000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '1', 'Corner', 120, 4500, 540000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '2', 'Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '3', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '4', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '5', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '6', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '7', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '8', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '9', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '10', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '11', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '12', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '13', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '14', 'Corner', 132, 4500, 594000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '4', '15', 'Corner', 110, 4500, 495000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '1', 'Corner', 119, 4500, 535500, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '2', 'Commercial / Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '3', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '4', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '5', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '6', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '7', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '8', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '9', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '10', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '11', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '12', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '13', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '14', 'Corner', 132, 4500, 594000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '5', '15', 'Corner', 110, 4500, 495000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '1', 'Corner', 116, 4500, 522000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '2', 'Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '3', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '4', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '5', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '6', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '7', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '8', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '9', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '10', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '11', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '12', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '13', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '14', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '15', 'Corner', 100, 4500, 450000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '16', 'Inner', 117, 4300, 503100, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '17', 'Inner', 100, 4300, 430000, 'Available'),
    ('11111111-1111-4111-8111-111111111111', '6', '18', 'Corner', 113, 4500, 508500, 'Available')
ON CONFLICT (project_id, block, lot) DO NOTHING;

-- Mark the lots that already have a buyer, then insert the buyer contracts
-- and an opening-balance payment reflecting what's already been paid.

WITH c AS (
  INSERT INTO public.contracts (
      project_id, lot_id, buyer_name, buyer_address, buyer_gmail, buyer_contact,
      lot_block, lot_lot, lot_area, lot_type, lot_rate,
      contract_price, payment_plan, initial_payment, term_years, monthly_amortization,
      due_day, next_due_date, approval_at, particulars
  )
  SELECT '11111111-1111-4111-8111-111111111111', l.id, 'Juan dela Cruz', '123 Sampaguita St., Barangay San Jose, Quezon City', 'juan.delacruz@gmail.com', '09171234567',
      '3', '12', 100, 'Inner', 4300,
      430000, 'installment', 0, 7, 5119.05,
      EXTRACT(DAY FROM DATE '2026-07-11')::INT, DATE '2026-07-11', DATE '2026-06-11', 'Regular payment'
  FROM public.lots l
  WHERE l.project_id = '11111111-1111-4111-8111-111111111111' AND l.block = '3' AND l.lot = '12'
  RETURNING id
)
INSERT INTO public.payments (contract_id, amount, method, months_covered, paid_at)
SELECT id, 102000, 'cash', 12, DATE '2026-06-11' FROM c;

UPDATE public.lots SET owner_buyer = 'Juan dela Cruz', status = 'Reserved'
WHERE project_id = '11111111-1111-4111-8111-111111111111' AND block = '3' AND lot = '12';

WITH c AS (
  INSERT INTO public.contracts (
      project_id, lot_id, buyer_name, buyer_address, buyer_gmail, buyer_contact,
      lot_block, lot_lot, lot_area, lot_type, lot_rate,
      contract_price, payment_plan, initial_payment, term_years, monthly_amortization,
      due_day, next_due_date, approval_at, particulars
  )
  SELECT '11111111-1111-4111-8111-111111111111', l.id, 'Maria Santos', '45 Rosal Ave., Barangay Poblacion, Makati City', 'maria.santos@gmail.com', '09281234567',
      '5', '7', 100, 'Inner', 4300,
      430000, 'installment', 0, 5, 7166.67,
      EXTRACT(DAY FROM DATE '2026-07-15')::INT, DATE '2026-07-15', DATE '2026-06-15', 'Advance payment – Dec 2025'
  FROM public.lots l
  WHERE l.project_id = '11111111-1111-4111-8111-111111111111' AND l.block = '5' AND l.lot = '7'
  RETURNING id
)
INSERT INTO public.payments (contract_id, amount, method, months_covered, paid_at)
SELECT id, 288000, 'cash', 24, DATE '2026-06-15' FROM c;

UPDATE public.lots SET owner_buyer = 'Maria Santos', status = 'Reserved'
WHERE project_id = '11111111-1111-4111-8111-111111111111' AND block = '5' AND lot = '7';

WITH c AS (
  INSERT INTO public.contracts (
      project_id, lot_id, buyer_name, buyer_address, buyer_gmail, buyer_contact,
      lot_block, lot_lot, lot_area, lot_type, lot_rate,
      contract_price, payment_plan, initial_payment, term_years, monthly_amortization,
      due_day, next_due_date, approval_at, particulars
  )
  SELECT '11111111-1111-4111-8111-111111111111', l.id, 'Roberto Reyes', '78 Malvar Road, Barangay Sto. Nino, Caloocan City', 'roberto.reyes@gmail.com', '09361234567',
      '1', '3', 115, 'Commercial / Corner', 4500,
      517500, 'installment', 0, 6, 7187.5,
      EXTRACT(DAY FROM DATE '2026-07-20')::INT, DATE '2026-07-20', DATE '2026-05-20', 'Missed – Nov 2025'
  FROM public.lots l
  WHERE l.project_id = '11111111-1111-4111-8111-111111111111' AND l.block = '1' AND l.lot = '3'
  RETURNING id
)
INSERT INTO public.payments (contract_id, amount, method, months_covered, paid_at)
SELECT id, 81600, 'cash', 12, DATE '2026-05-20' FROM c;

UPDATE public.lots SET owner_buyer = 'Roberto Reyes', status = 'Reserved'
WHERE project_id = '11111111-1111-4111-8111-111111111111' AND block = '1' AND lot = '3';
