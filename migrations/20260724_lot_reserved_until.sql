ALTER TABLE public.lots
  ADD COLUMN IF NOT EXISTS reserved_until TIMESTAMPTZ NULL;

-- Contracted installment buyers were marked Reserved; they are Sold.
UPDATE public.lots l
SET status = 'Sold',
    reserved_until = NULL
WHERE l.status = 'Reserved'
  AND EXISTS (
      SELECT 1 FROM public.contracts c WHERE c.lot_id = l.id
  );
