/**
 * Removes all 30-month commission split history rows and resets affected
 * contracts.agent_commission_split_months to their pre-change (genesis) value.
 *
 * Usage: NODE_TLS_REJECT_UNAUTHORIZED=0 node scripts/reset-30-month-splits.mjs
 */
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

const client = new pg.Client({
    connectionString: databaseUrl,
    ssl: databaseUrl.includes("supabase") ? { rejectUnauthorized: false } : undefined,
});

await client.connect();
try {
    const preview = await client.query(`
      SELECT h.id, h.contract_id, c.buyer_name, h.split_months,
             h.effective_period_start::text AS eff, h.rebalance_strategy
        FROM public.contract_split_history h
        JOIN public.contracts c ON c.id = h.contract_id
       WHERE h.split_months = 30
       ORDER BY c.buyer_name
    `);
    console.log(`Will delete ${preview.rows.length} history row(s) with split_months = 30:`);
    for (const row of preview.rows) {
        console.log(`  - ${row.buyer_name} (${row.eff}, ${row.rebalance_strategy})`);
    }

    const flatOnly = await client.query(`
      SELECT c.id, c.buyer_name, c.agent_commission_split_months
        FROM public.contracts c
       WHERE c.agent_commission_split_months = 30
         AND c.selling_agent_id IS NOT NULL
         AND NOT EXISTS (
           SELECT 1 FROM public.contract_split_history h
            WHERE h.contract_id = c.id AND h.split_months = 30
         )
       ORDER BY c.buyer_name
    `);
    if (flatOnly.rows.length > 0) {
        console.log(`\nWill reset flat column (no 30-mo history) for ${flatOnly.rows.length} contract(s):`);
        for (const row of flatOnly.rows) {
            console.log(`  - ${row.buyer_name}`);
        }
    }

    await client.query("BEGIN");

    const deleted = await client.query(`
      DELETE FROM public.contract_split_history
       WHERE split_months = 30
      RETURNING contract_id
    `);
    const affectedIds = [...new Set(deleted.rows.map((r) => r.contract_id))];
    console.log(`\nDeleted ${deleted.rowCount} history row(s) across ${affectedIds.length} contract(s).`);

    const now = Math.floor(Date.now() / 1000);

    for (const contractId of affectedIds) {
        const genesis = await client.query(
            `SELECT split_months FROM public.contract_split_history
              WHERE contract_id = $1
           ORDER BY effective_period_start ASC
              LIMIT 1`,
            [contractId],
        );
        const restoreMonths = genesis.rows[0]?.split_months ?? 20;
        await client.query(
            `UPDATE public.contracts
                SET agent_commission_split_months = $1, updated_at = $2
              WHERE id = $3`,
            [restoreMonths, now, contractId],
        );
    }

    for (const row of flatOnly.rows) {
        const genesis = await client.query(
            `SELECT split_months FROM public.contract_split_history
              WHERE contract_id = $1
           ORDER BY effective_period_start ASC
              LIMIT 1`,
            [row.id],
        );
        const restoreMonths = genesis.rows[0]?.split_months ?? 20;
        await client.query(
            `UPDATE public.contracts
                SET agent_commission_split_months = $1, updated_at = $2
              WHERE id = $3`,
            [restoreMonths, now, row.id],
        );
    }

    await client.query("COMMIT");

    const verify = await client.query(`
      SELECT
        (SELECT COUNT(*)::int FROM public.contract_split_history WHERE split_months = 30) AS history_30,
        (SELECT COUNT(*)::int FROM public.contracts WHERE agent_commission_split_months = 30 AND selling_agent_id IS NOT NULL) AS flat_30
    `);
    console.log("\nAfter reset:", verify.rows[0]);
    console.log("Done — re-apply splits manually via Commission Split in the UI.");
} catch (err) {
    await client.query("ROLLBACK").catch(() => undefined);
    console.error(err);
    process.exit(1);
} finally {
    await client.end();
}
