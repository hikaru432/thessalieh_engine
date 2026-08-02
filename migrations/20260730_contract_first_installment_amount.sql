-- Optional per-contract override for installment #1's expected amount (the
-- "Preferred amount" option in AddClientDetails). NULL = installment #1 uses
-- monthly_amortization like every other slot (existing behavior, also how the
-- "Half payment" option stays represented — see AddClientDetails.tsx /
-- tracking/trackingHelpers.ts expectedInstallmentAmount).

ALTER TABLE public.contracts ADD COLUMN IF NOT EXISTS first_installment_amount double precision;
