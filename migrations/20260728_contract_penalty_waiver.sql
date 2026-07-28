-- Lets admin waive the 3%/month late penalty for a buyer's current unpaid
-- installment. NULL = no waiver. When set, any unpaid installment due on or
-- before this date is exempt from the penalty computation (see
-- compute_next_unpaid_due_date / tracking/trackingHelpers.ts computeInstallmentPenalty).

ALTER TABLE public.contracts
  ADD COLUMN IF NOT EXISTS penalty_waived_through_due_date date;
