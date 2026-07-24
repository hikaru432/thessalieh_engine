-- Promo TCP on contracts: catalog list_price + effective contract_price.

ALTER TABLE public.contracts
  ADD COLUMN IF NOT EXISTS is_promo boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS list_price double precision NOT NULL DEFAULT 0;

UPDATE public.contracts
SET list_price = contract_price
WHERE list_price = 0 AND contract_price > 0;
