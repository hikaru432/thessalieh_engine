# Thessalieh Engine — Updates

Changelog of product and API updates from the initial Auth + Project baseline through the current marketing, commissions, and cashflow work.

---

## Timeline (committed)

| Commit | Date | Summary |
|--------|------|---------|
| `bb250bd` | 2026-07-07 | **Auth + Project** — engine bootstrap, users (register/login/session/password reset), company, projects, Ed25519 verified envelopes, CSRF, rate limits |
| `83c91dc` | 2026-07-07 | **Lots** — project lot CRUD |
| `525bd24` | 2026-07-07 | **Contract Structure** — contracts + payments under projects |
| `de69faf` | 2026-07-07 | **Route Separation + Payload Limits** — route layout and body size limits |
| `34156bd` | 2026-07-08 | **Auth Rework** — simplified register/login; removed separate verify flow |
| `5d3d2c5` | 2026-07-14 | **Admin** — moved company/projects/lots/contracts under `admin`; added roster + commission rates + user list |

Work below that commit (migrations, commission tracking, auth role model, cashflow) is the current uncommitted line of updates.

---

## Overview (current line)

This release expands the admin surface for **marketing / roster**, **TCP commission allocation**, **biweek commission tracking**, **contract buyer accounts + promo pricing**, and **payment cashflow**. Auth invite tokens no longer grant Admin; operational roles sync from the Marketing roster.

---

## 1. Auth and users

### Behavior change (breaking)

| Before | After |
|--------|--------|
| Valid `access_token` on register → role **Admin** | Token is validated/redeemed; role is always **User** |
| Admin only via invite | Bootstrap Admin with a one-time SQL `UPDATE`; LB / TO / Agent via roster sync |

### Shared insert path

- New [`src/api/users/insert.rs`](../src/api/users/insert.rs) — Argon2 hash, invite redeem, **always `role = "User"`**
- [`register.rs`](../src/api/users/register.rs) — thin wrapper that also sets session cookies
- New [`create.rs`](../src/api/users/create.rs) — admin `POST /users` (create without cookies)

### List users

- `GET /users` returns `role` and `phone`
- Optional filter: `?role=`

### Bootstrap CLI

- [`src/scripts/register-cli.mjs`](../src/scripts/register-cli.mjs) — signed `POST /auth/register` (Ed25519) when there is no signup UI
- Usage (from repo root, API on `:8080`):

```powershell
node src/scripts/register-cli.mjs -u <username> -p <password> -t <access_token>
```

- `-a` is optional API **base** URL only (default `http://localhost:8080`), not a full `/auth/register` path
- After register, promote Admin manually if needed:

```sql
UPDATE public.users SET role = 'Admin' WHERE username = '<admin_username>';
```

Local cheat sheet: `generate.txt` (contains secrets — do not commit as-is).

---

## 2. Roles and roster

### Allowed `users.role` values

`User` | `Admin` | `Lead Broker` | `Titling Officer` | `Agent`

### Roster sync

On roster create / update / delete:

- `users.role` is set to match the roster role
- On remove or reassignment, the previous user’s role reverts to `User`
- **`Admin` is never overwritten** by roster sync

### Roster rules

- Agents require an upline; upline must be Lead Broker or Titling Officer
- Lead Broker / Titling Officer: commission rate forced to `0` (company baseline rates apply)
- Agents: commission capped by the Agent pool rate in `commission_rates`

Admin APIs still require `users.role = 'Admin'` (`require_admin`).

---

## 3. Commission rates

Table `commission_rates` (role PK), seeded / editable for:

| Role | Default (seed) |
|------|----------------|
| Lead Broker | 5% |
| Titling Officer | 3% |
| Agent | 12% |
| Legal Counsel | 5% |
| Land Owner | 40% |
| Hypomone | 25% |
| Project Dev & Processing | 10% |

`PATCH` upserts on `(role)` (`ON CONFLICT`).

---

## 4. Projects / marketing

### Schema

| Column | Purpose |
|--------|---------|
| `lead_broker_roster_id` | FK to roster |
| `titling_officer_roster_id` | FK to roster |
| `agent_commission_split_months` | Default **15** |
| `agents_json` | JSONB array of agent objects (default `[]`) |

### API

- Project responses include the fields above
- Create initializes split months `15` and `agents_json = []`
- **New:** `PATCH /projects/{project_id}/agents` — each agent object must include `id`, `name`, `role`

---

## 5. Company settings

- `company_settings.agent_commission_split_months` (default **15**, range 1–120)
- Included in company settings GET/PATCH
- **New:** `PATCH /company/agent-commission-split-months`

---

## 6. Commission tracking (new)

### Period status — `commission_period_status`

Per project / subject / biweek:

- Status: `not_yet` | `partial` | `pending` | `paid`
- Partial amount / date when applicable

| Method | Path |
|--------|------|
| `GET` | `/projects/{project_id}/commission-status?from=&to=` |
| `PUT` | `/projects/{project_id}/commission-status` |

### Row meta — `commission_row_meta`

Per buyer / agent row on a biweek:

- `other_flag`: `none` | `half` | `full` (biweek release / half amort flags)

| Method | Path |
|--------|------|
| `GET` | `/projects/{project_id}/commission-row-meta?subject_agent_id=` |
| `PUT` | `/projects/{project_id}/commission-row-meta` |

---

## 7. Contracts

### Buyer account (breaking)

- Create requires `buyer_user_id` — user must have role `User`
- One buyer account per project
- Structured names required: `buyer_last_name`, `buyer_first_name`, `buyer_middle_name`
- Responses can include `buyer_username`

### Pricing

- `is_promo`, `list_price`, effective `contract_price`
- Promo TCP must be ≤ catalog price (`area × rate`)

### Terms and split

- `term_months` (alongside `term_years`)
- Per-contract `agent_commission_split_months` (API default **36**, range 1–120; migration backfills from company setting)

---

## 8. Payments / cashflow

### Methods (breaking)

Allowed: `cash` | `gcash` | `maya` | `bank` | `others`

- Legacy `card` migrated to `others` with `mode_label` / metadata
- Opening payment and `record_payment` validate reference / bank / sender / receiver / mode_label as required by method

### API

- **New:** `GET /projects/{project_id}/payments` — project-wide cashflow list

---

## 9. Lots

| Change | Detail |
|--------|--------|
| Status `Installment` | Unpaid installment / half amort after contract |
| `reserved_until` | Unix timestamp; required and future when status is `Reserved`; cleared otherwise |
| Status semantics | Contracted unpaid installment/half → **Installment**; fully paid → **Sold**; **Reserved** only for timed holds |

---

## 10. Migrations (apply in date order)

| File | Change |
|------|--------|
| `20260723_commission_rates_seed.sql` | Create/seed `commission_rates` |
| `20260723_project_marketing.sql` | Project LB/TO FKs + split months; re-seed TCP rates |
| `20260723_company_agent_split_months.sql` | Company split months |
| `20260723_buyer_name_parts.sql` | Contract buyer name parts + backfill |
| `20260723_term_months.sql` | `term_months` + backfill from years |
| `20260724_users_role_expand.sql` | Role CHECK: User / Admin / LB / TO; sync from roster |
| `20260724_users_role_add_agent.sql` | Add Agent; sync non-Admin roster roles |
| `20260724_project_agents_json.sql` | `projects.agents_json` |
| `20260724_contract_agent_split_months.sql` | Contract split months + backfill |
| `20260724_contract_buyer_user_id.sql` | `buyer_user_id` FK + index |
| `20260724_contract_promo_tcp.sql` | `is_promo`, `list_price` |
| `20260724_lot_reserved_until.sql` | `reserved_until`; Reserved+contract → Sold |
| `20260724_lot_status_installment.sql` | Status `Installment` + backfill |
| `20260724_payments_cashflow.sql` | Payment meta; method enum without `card` |
| `20260724_commission_period_status.sql` | New period status table |
| `20260724_commission_row_meta.sql` | New row meta table |

Optional Node runners under `scripts/` for selected migrations and roster checks (`run-*-migration.mjs`, `check-role-sync.mjs`, `inspect-roster.mjs`, seed helpers). Prefer applying the SQL files directly in production.

---

## 11. New / extended API surface

| Method | Path | Notes |
|--------|------|-------|
| `POST` | `/users` | Admin create user |
| `PATCH` | `/company/agent-commission-split-months` | Dedicated company split months |
| `PATCH` | `/projects/{project_id}/agents` | Project agents JSON |
| `GET` / `PUT` | `/projects/{project_id}/commission-status` | Biweek period status |
| `GET` / `PUT` | `/projects/{project_id}/commission-row-meta` | Per-row commission flags |
| `GET` | `/projects/{project_id}/payments` | Project cashflow |

Existing admin routes for company, projects, lots, contracts, roster, and commission rates keep their paths; payloads are expanded as described above.

---

## 12. Breaking changes checklist

1. **Invite token ≠ Admin** — promote with SQL or future tooling; assign LB/TO/Agent via Marketing roster.
2. **Contracts require `buyer_user_id` + name parts** — name-only create is rejected.
3. **Payment method `card` removed** — use `others` (+ `mode_label`).
4. **Lot statuses** — unpaid installment/half → `Installment`; `Reserved` needs future `reserved_until`.
5. **User list** — includes `role` / `phone`.
6. **Split months defaults** — company/project default **15**; contract API default **36** (migration backfills contracts from company).

---

## 13. Suggested bootstrap flow

1. Insert an `access_tokens` row; register the first user via `register-cli.mjs`.
2. `UPDATE users SET role = 'Admin'` for that username.
3. Register brokers / officers / agents as normal users (invite optional).
4. Admin → Marketing → Broker & agent roster — assign Lead Broker, Titling Officer, Agent (syncs `users.role`).
5. Assign LB/TO on projects; set `agents_json` / commission rates / split months as needed.
6. Create buyer `User` accounts before contracts; record payments with the new method set.
