ALTER TABLE public.contracts
  ADD COLUMN IF NOT EXISTS buyer_last_name  TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS buyer_first_name TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS buyer_middle_name TEXT NOT NULL DEFAULT '';

UPDATE public.contracts
   SET buyer_last_name = buyer_name
 WHERE buyer_last_name = '' AND buyer_name <> '';
