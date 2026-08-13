import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import pg from "pg";

const root = path.dirname(fileURLToPath(import.meta.url));
const envPath = path.join(root, "..", ".env");
const envText = fs.readFileSync(envPath, "utf8");
const databaseUrl = envText
    .split("\n")
    .map((line) => line.trim())
    .find((line) => line.startsWith("DATABASE_URL="))
    ?.slice("DATABASE_URL=".length);

if (!databaseUrl) {
    console.error("DATABASE_URL not found in .env");
    process.exit(1);
}

const migrationPath = path.join(root, "..", "migrations", "20260813_project_rate_config.sql");
const sql = fs.readFileSync(migrationPath, "utf8");

// Strip sslmode from the URL — newer pg-connection-string treats sslmode=require as an
// alias for verify-full, which overrides the explicit `ssl` option below and fails on
// Supabase's pooler cert chain. The `ssl` option here is the actual source of truth.
const connectionStringNoSslMode = databaseUrl.replace(/([?&])sslmode=[^&]*&?/, "$1").replace(/[?&]$/, "");

const client = new pg.Client({
    connectionString: connectionStringNoSslMode,
    ssl: databaseUrl.includes("supabase") ? { rejectUnauthorized: false } : undefined,
});

await client.connect();
try {
    await client.query(sql);

    const categories = await client.query(`
      SELECT project_id, label, percent, is_agent_pool, sort_order
      FROM public.project_rate_categories
      ORDER BY project_id, sort_order ASC
    `);
    const uplineRates = await client.query(`
      SELECT project_id, upline_role_type_slug, percent
      FROM public.project_upline_role_rates
      ORDER BY project_id, upline_role_type_slug ASC
    `);

    console.log("Migration applied.");
    console.log(`project_rate_categories: ${categories.rowCount} rows`);
    console.log(`project_upline_role_rates: ${uplineRates.rowCount} rows`);
} finally {
    await client.end();
}
