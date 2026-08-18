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

const client = new pg.Client({
    connectionString: databaseUrl,
    ssl: databaseUrl.includes("supabase") ? { rejectUnauthorized: false } : undefined,
});

function periodStartContaining(dateYmd) {
    const [y, m, d] = dateYmd.split("-").map(Number);
    const day = d <= 15 ? 1 : 16;
    return `${y}-${String(m).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
}

await client.connect();
try {
    const gaps = await client.query(`
      SELECT c.id, c.buyer_name, c.agent_commission_split_months, c.approval_at, c.updated_at,
             (SELECT COUNT(*)::int FROM public.contract_split_history h WHERE h.contract_id = c.id) AS history_rows
        FROM public.contracts c
       WHERE c.agent_commission_split_months = 30
         AND c.selling_agent_id IS NOT NULL
         AND NOT EXISTS (
           SELECT 1 FROM public.contract_split_history h
            WHERE h.contract_id = c.id AND h.split_months = 30
         )
       ORDER BY c.buyer_name
    `);
    console.log("30-mo flat but no 30-mo history row:", JSON.stringify(gaps.rows, null, 2));
} finally {
    await client.end();
}
