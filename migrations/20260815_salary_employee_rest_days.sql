-- Per-employee rest days (0=Sunday..6=Saturday, matching JS Date.getUTCDay()) — used
-- to compute a daily rate when a pay plan only partially covers a monthly/semimonthly/
-- weekly period (e.g. training ends mid-month, salary starts mid-month), and lays the
-- groundwork for a future attendance-scanner integration. Empty array = no rest days
-- configured, i.e. every calendar day counts as a working day.

ALTER TABLE public.salary_employees
    ADD COLUMN IF NOT EXISTS rest_days SMALLINT[] NOT NULL DEFAULT '{}';
