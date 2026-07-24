ALTER TABLE public.contracts
  ADD COLUMN IF NOT EXISTS buyer_user_id UUID REFERENCES public.users(id);

CREATE INDEX IF NOT EXISTS idx_contracts_buyer_user_id ON public.contracts(buyer_user_id);
