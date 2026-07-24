-- Cash Flow payment modes + metadata (no external wallet APIs).

ALTER TABLE public.payments
  ADD COLUMN IF NOT EXISTS reference_no text NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS bank_name text NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS sender_name text NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS receiver_name text NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS mode_label text NOT NULL DEFAULT '';

UPDATE public.payments
SET method = 'others',
    mode_label = CASE WHEN COALESCE(mode_label, '') = '' THEN 'Card' ELSE mode_label END
WHERE method = 'card';

ALTER TABLE public.payments DROP CONSTRAINT IF EXISTS payments_method_check;

ALTER TABLE public.payments
  ADD CONSTRAINT payments_method_check
  CHECK (method IN ('cash', 'gcash', 'maya', 'bank', 'others'));
