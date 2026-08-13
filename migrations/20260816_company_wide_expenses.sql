-- Expenses become company-wide instead of per-project — Salary Records and
-- Position types were already company-wide (see 20260813b_salary_system.sql /
-- employee_position_types); this brings Expenses in line so all three sit under
-- one company-level Cash Out view instead of being duplicated per project. The
-- per-project money-out ledgers (commission_release_entries, salary_release_entries)
-- are unaffected — those stay tied to a project on purpose.

ALTER TABLE public.expense_categories DROP CONSTRAINT IF EXISTS expense_categories_project_id_name_key;
DROP INDEX IF EXISTS public.expense_categories_project_idx;
DROP INDEX IF EXISTS public.expenses_project_category_idx;

ALTER TABLE public.expense_categories DROP COLUMN IF EXISTS project_id;
ALTER TABLE public.expenses DROP COLUMN IF EXISTS project_id;

-- Two different projects were free to each have their own "Refund"/"Salary"/etc.
-- category under the old per-project uniqueness — now that name alone must be
-- unique, collapse same-named categories into one before the constraint below can
-- even be added: keep the oldest row per name, repoint every expense that pointed
-- at a duplicate onto the surviving row, then drop the duplicates.
WITH canonical AS (
    SELECT name, MIN(id) AS keep_id
      FROM public.expense_categories
  GROUP BY name
),
remap AS (
    SELECT ec.id AS old_id, c.keep_id
      FROM public.expense_categories ec
      JOIN canonical c ON c.name = ec.name
     WHERE ec.id <> c.keep_id
)
UPDATE public.expenses e
   SET category_id = r.keep_id
  FROM remap r
 WHERE e.category_id = r.old_id;

DELETE FROM public.expense_categories ec
 USING (
     SELECT name, MIN(id) AS keep_id
       FROM public.expense_categories
   GROUP BY name
 ) c
 WHERE ec.name = c.name AND ec.id <> c.keep_id;

ALTER TABLE public.expense_categories ADD CONSTRAINT expense_categories_name_key UNIQUE (name);
CREATE INDEX IF NOT EXISTS expenses_category_idx ON public.expenses (category_id);
