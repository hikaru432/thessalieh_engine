-- Allow Installment as a lot status for unpaid installment/half contracts.
-- Flip Sold lots that still have an unpaid installment/half contract back to Installment.

ALTER TABLE public.lots DROP CONSTRAINT IF EXISTS lots_status_check;

ALTER TABLE public.lots
  ADD CONSTRAINT lots_status_check
  CHECK (status IN ('Available', 'Hold', 'Reserved', 'Installment', 'Sold'));

UPDATE public.lots l
SET status = 'Installment',
    reserved_until = NULL
WHERE l.status = 'Sold'
  AND EXISTS (
      SELECT 1
        FROM public.contracts c
       WHERE c.lot_id = l.id
         AND c.payment_plan IN ('installment', 'half')
         AND COALESCE(
               (SELECT SUM(p.amount) FROM public.payments p WHERE p.contract_id = c.id),
               0
             ) < c.contract_price
  );
