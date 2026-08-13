-- Site plan map geometry for the subdivision editor (blocks, dividing lines, lot slots).
-- Lot business data remains in public.lots; this JSON stores layout only.

ALTER TABLE public.projects
    ADD COLUMN IF NOT EXISTS subdivision_layout JSONB NULL;

COMMENT ON COLUMN public.projects.subdivision_layout IS
    'Subdivision editor layout: project boundary, blocks, row/col lines, lot slot positions.';
