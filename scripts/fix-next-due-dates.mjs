// One-time backfill: recompute `contracts.next_due_date` for every contract using
// the corrected installment-schedule math (see docs/UPDATES.md and
// src/api/admin/contracts.rs `compute_next_unpaid_due_date`). Fixes contracts whose
// next_due_date was seeded before that fix landed — previously a buyer whose first
// monthly amortization was paid on the approval date could get an installment 2 due
// only days later (same month) instead of a full month out, and every payment after
// that inherited the drift since record_payment used to just add months to whatever
// was already stored.
//
// Mirrors thessalieh-property/src/pages/admin/tracking/trackingHelpers.ts
// (installmentDueDate / nextUnpaidDueDate) — keep all three in sync.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import pg from "pg";

const root = path.dirname(fileURLToPath(import.meta.url));
const envText = fs.readFileSync(path.join(root, "..", ".env"), "utf8");
const databaseUrl = envText
    .split("\n")
    .map((line) => line.trim())
    .find((line) => line.startsWith("DATABASE_URL="))
    ?.slice("DATABASE_URL=".length);

if (!databaseUrl) {
    console.error("DATABASE_URL not found in .env");
    process.exit(1);
}

// Keep DATE columns as plain 'YYYY-MM-DD' strings — avoids any local-timezone
// shifting that node-postgres's default Date parsing could introduce.
pg.types.setTypeParser(1082, (value) => value);

function parseYmd(value) {
    if (!value) return null;
    const [y, m, d] = value.split("-").map(Number);
    return new Date(Date.UTC(y, (m || 1) - 1, d || 1));
}

function toYmd(date) {
    const y = date.getUTCFullYear();
    const m = String(date.getUTCMonth() + 1).padStart(2, "0");
    const d = String(date.getUTCDate()).padStart(2, "0");
    return `${y}-${m}-${d}`;
}

function lastDayOfMonth(year, month1) {
    return new Date(Date.UTC(year, month1, 0)).getUTCDate();
}

function monthIndex(date) {
    return date.getUTCFullYear() * 12 + date.getUTCMonth();
}

function dateAtMonthIndex(totalMonthIndex, dueDay) {
    const year = Math.floor(totalMonthIndex / 12);
    const month0 = ((totalMonthIndex % 12) + 12) % 12;
    const last = lastDayOfMonth(year, month0 + 1);
    const day = Math.min(Math.max(1, dueDay), last);
    return new Date(Date.UTC(year, month0, day));
}

function installmentDueDate(anchor, dueDay, initialPayment, installmentIndex) {
    const day = Math.min(31, Math.max(1, dueDay));
    const firstAmortOnApproval = (initialPayment ?? 0) <= 0;

    if (firstAmortOnApproval && installmentIndex === 1) return anchor;

    if (firstAmortOnApproval) {
        const idx = monthIndex(anchor) + 1 + (installmentIndex - 2);
        return dateAtMonthIndex(idx, day);
    }

    const firstDueIdx =
        day > anchor.getUTCDate() ? monthIndex(anchor) : monthIndex(anchor) + 1;
    return dateAtMonthIndex(firstDueIdx + (installmentIndex - 1), day);
}

function buildSchedule(anchor, dueDay, initialPayment, totalMonths) {
    if (totalMonths <= 0) return [];
    const out = [];
    for (let i = 1; i <= totalMonths; i++) {
        out.push(installmentDueDate(anchor, dueDay, initialPayment, i));
    }
    return out;
}

function isOpeningDownPayment(initialPayment, approvalAt, amount, paidAt) {
    return (
        initialPayment > 0 &&
        Math.abs(amount - initialPayment) < 1e-6 &&
        approvalAt &&
        paidAt &&
        approvalAt.getTime() === paidAt.getTime()
    );
}

function isPaidAmount(amount, monthlyAmort) {
    return monthlyAmort <= 0 ? amount > 0 : amount >= monthlyAmort * 0.95;
}

function computeNextUnpaidDueDate(anchor, dueDay, initialPayment, approvalAt, monthlyAmort, totalMonths, payments) {
    const schedule = buildSchedule(anchor, dueDay, initialPayment, totalMonths);
    if (schedule.length === 0) return null;

    const paid = schedule.map(() => false);
    const recurring = payments
        .filter((p) => !isOpeningDownPayment(initialPayment, approvalAt, p.amount, p.paidAt))
        .slice()
        .sort((a, b) => a.paidAt - b.paidAt);

    let cursor = 0;
    for (const p of recurring) {
        const covers = p.monthsCovered === 0 ? 1 : Math.max(1, p.monthsCovered);
        const per = p.amount / covers;
        for (let i = 0; i < covers && cursor < paid.length; i++) {
            if (isPaidAmount(per, monthlyAmort)) paid[cursor] = true;
            cursor += 1;
        }
    }

    const idx = paid.findIndex((v) => !v);
    return idx === -1 ? null : schedule[idx];
}

// Strip ?sslmode=require etc. — pg's own SSL parsing from the query string can
// conflict with the explicit `ssl` option below and reject Supabase's chain.
const connectionString = databaseUrl.replace(/\?.*$/, "");
const client = new pg.Client({
    connectionString,
    ssl: databaseUrl.includes("supabase") ? { rejectUnauthorized: false } : undefined,
});

await client.connect();
try {
    const contracts = await client.query(`
        SELECT id, buyer_name, due_day, initial_payment, approval_at, amort_start_date,
               monthly_amortization, term_years, term_months, next_due_date, updated_at
          FROM public.contracts
      ORDER BY created_at ASC
    `);

    const changes = [];
    let skippedNoAnchor = 0;

    for (const c of contracts.rows) {
        const anchor =
            parseYmd(c.amort_start_date) ??
            parseYmd(c.approval_at) ??
            (c.updated_at ? new Date(Number(c.updated_at) * 1000) : null);
        if (!anchor) {
            skippedNoAnchor += 1;
            continue;
        }

        const totalMonths =
            c.term_months > 0 ? c.term_months : c.term_years > 0 ? c.term_years * 12 : 1;

        const paymentsRes = await client.query(
            `SELECT amount, paid_at, months_covered FROM public.payments WHERE contract_id = $1`,
            [c.id],
        );
        const payments = paymentsRes.rows.map((p) => ({
            amount: Number(p.amount),
            paidAt: parseYmd(p.paid_at),
            monthsCovered: Number(p.months_covered),
        }));

        const corrected = computeNextUnpaidDueDate(
            anchor,
            c.due_day,
            Number(c.initial_payment),
            parseYmd(c.approval_at),
            Number(c.monthly_amortization),
            totalMonths,
            payments,
        );
        const correctedStr = corrected ? toYmd(corrected) : c.next_due_date;

        if (correctedStr !== c.next_due_date) {
            changes.push({
                id: c.id,
                buyer: c.buyer_name,
                from: c.next_due_date,
                to: correctedStr,
            });
            await client.query(`UPDATE public.contracts SET next_due_date = $1 WHERE id = $2`, [
                correctedStr,
                c.id,
            ]);
        }
    }

    console.log(`Checked ${contracts.rows.length} contract(s).`);
    if (skippedNoAnchor > 0) {
        console.log(`Skipped ${skippedNoAnchor} contract(s) with no approval date, amort start date, or updated_at.`);
    }
    console.log(`Updated ${changes.length} contract(s):`);
    console.table(changes);
} finally {
    await client.end();
}
