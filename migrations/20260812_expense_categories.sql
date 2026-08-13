-- Project-scoped expense tracking: an admin defines categories (Refund, Salary, etc.)
-- for a project, then records individual expense line items under a category. Together
-- with commission releases, this forms the "Cash Out" side of the Dashboard/Transactions
-- ledgers (commission releases already counted as Cash Out before this).

CREATE TABLE IF NOT EXISTS public.expense_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0,
    UNIQUE (project_id, name)
);

CREATE TABLE IF NOT EXISTS public.expenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES public.projects(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES public.expense_categories(id) ON DELETE CASCADE,
    paid_to TEXT NOT NULL,
    description TEXT,
    amount DOUBLE PRECISION NOT NULL CHECK (amount > 0),
    paid_at DATE NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS expense_categories_project_idx ON public.expense_categories (project_id);
CREATE INDEX IF NOT EXISTS expenses_project_category_idx ON public.expenses (project_id, category_id);
