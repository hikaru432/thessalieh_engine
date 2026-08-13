-- Dynamic Salary System: a dedicated employee list (separate from the commission-based
-- Agent/Lead Broker/Titling Officer roster), a versioned pay-plan history per employee
-- (training lump-sum window, then an ongoing monthly salary on a chosen release
-- cadence), and a project-scoped release ledger mirroring commission_release_entries
-- so a release rolls into that project's Cash Out.

CREATE TABLE IF NOT EXISTS public.salary_employees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    position TEXT,
    status TEXT NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
    created_at BIGINT NOT NULL DEFAULT 0
);

-- Append-only pay-plan history per employee, resolved the same way
-- commission_split_schedule is (see resolveDefaultSplitMonthsForDate): the latest row
-- with start_date <= target date wins. 'training' rows are a flat one-time fee for an
-- explicit [start_date, end_date] window. 'regular' rows are open-ended (end_date NULL)
-- until superseded by a later row's start_date, carrying an ongoing monthly_amount +
-- release schedule_type.
CREATE TABLE IF NOT EXISTS public.salary_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id UUID NOT NULL REFERENCES public.salary_employees(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('training', 'regular')),
    start_date DATE NOT NULL,
    end_date DATE,
    training_fee DOUBLE PRECISION CHECK (training_fee IS NULL OR training_fee > 0),
    monthly_amount DOUBLE PRECISION CHECK (monthly_amount IS NULL OR monthly_amount > 0),
    schedule_type TEXT CHECK (schedule_type IN ('weekly', 'semimonthly', 'monthly')),
    created_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (employee_id, start_date),
    CHECK (
        (kind = 'training' AND end_date IS NOT NULL AND training_fee IS NOT NULL
            AND monthly_amount IS NULL AND schedule_type IS NULL)
        OR
        (kind = 'regular' AND end_date IS NULL AND monthly_amount IS NOT NULL
            AND schedule_type IS NOT NULL AND training_fee IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS salary_plans_employee_idx ON public.salary_plans (employee_id, start_date);

-- Project-scoped release ledger, same shape as commission_release_entries: an admin can
-- record more than one dated amount against an employee + period (e.g. a partial
-- release now, another later); the frontend sums these to compute what's remaining.
CREATE TABLE IF NOT EXISTS public.salary_release_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    employee_id UUID NOT NULL REFERENCES public.salary_employees(id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    amount DOUBLE PRECISION NOT NULL CHECK (amount > 0),
    paid_at DATE NOT NULL,
    note TEXT,
    created_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS salary_release_entries_project_idx
    ON public.salary_release_entries (project_id, employee_id, period_start);
