-- Effective-dated agent commission split-month policy (e.g. "20 months before Aug 1
-- 2026, 30 months from Aug 1 2026 onward"). Only affects the *default* value new
-- contracts are created with — each contract still freezes its own
-- agent_commission_split_months at creation time and is never retroactively changed.
-- Safe to re-run: creates table if missing, seeds a baseline row only when the table
-- is empty (so a lookup always resolves to something).

CREATE TABLE IF NOT EXISTS public.commission_split_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    effective_date DATE NOT NULL UNIQUE,
    split_months INT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL DEFAULT 0
);

INSERT INTO public.commission_split_schedule (effective_date, split_months, created_at, updated_at)
SELECT '2000-01-01', 36, extract(epoch from now())::bigint, extract(epoch from now())::bigint
 WHERE NOT EXISTS (SELECT 1 FROM public.commission_split_schedule);
