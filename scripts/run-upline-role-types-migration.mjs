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

const migrationPath = path.join(root, "..", "migrations", "20260727_upline_role_types.sql");
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

    const verify = await client.query(`
      SELECT slug, label, base_commission_percent, portal_path, sort_order
      FROM public.upline_role_types
      ORDER BY sort_order ASC
    `);

    console.log("Migration applied.");
    console.log(JSON.stringify(verify.rows, null, 2));
} finally {
    await client.end();
}
