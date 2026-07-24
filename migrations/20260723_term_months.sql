ALTER TABLE public.contracts
  ADD COLUMN IF NOT EXISTS term_months INTEGER NOT NULL DEFAULT 0;

UPDATE public.contracts
   SET term_months = term_years * 12
 WHERE term_months = 0 AND term_years > 0;
