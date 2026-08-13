-- Employee Portal: an admin-managed list of position labels (Executive Secretary,
-- Janitor, etc.) for salary_employees.position, plus a nullable link from a
-- salary_employees row to a login account. Linking syncs users.role to "Employee"
-- the same way roster.user_id syncs Agent/Lead Broker/Titling Officer (see
-- sync_user_role/revert_user_role in src/api/admin/roster.rs), letting that account
-- reach a self-service /employee portal for its own pay plan + salary status.

CREATE TABLE IF NOT EXISTS public.employee_position_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    label TEXT NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX IF NOT EXISTS employee_position_types_label_unique
    ON public.employee_position_types (label);

ALTER TABLE public.salary_employees
    ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES public.users(id) ON DELETE SET NULL;

-- Partial unique index (not a column-level UNIQUE) so multiple employees can each
-- have a NULL user_id (no portal access) while still enforcing one employee per
-- linked account.
CREATE UNIQUE INDEX IF NOT EXISTS salary_employees_user_id_unique
    ON public.salary_employees (user_id) WHERE user_id IS NOT NULL;
