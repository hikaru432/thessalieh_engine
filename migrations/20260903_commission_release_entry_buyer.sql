-- Buyer-tagged commission releases: a cell click on one buyer records the
-- same period ledger amount, plus who it was for and an optional note.
-- Footer/modal releases leave these NULL.

ALTER TABLE public.commission_release_entries
    ADD COLUMN IF NOT EXISTS row_key TEXT,
    ADD COLUMN IF NOT EXISTS buyer_label TEXT,
    ADD COLUMN IF NOT EXISTS note TEXT;
