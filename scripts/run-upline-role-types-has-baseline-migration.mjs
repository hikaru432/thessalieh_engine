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

const connectionStringNoSslMode = databaseUrl.replace(/([?&])sslmode=[^&]*&?/, "$1").replace(/[?&]$/, "");

const client = new pg.Client({
    connectionString: connectionStringNoSslMode,
    ssl: databaseUrl.includes("supabase") ? { rejectUnauthorized: false } : undefined,
    connectionTimeoutMillis: 15000,
});

await client.connect();
try {
    const name = "20260806_upline_role_types_has_baseline.sql";
    const sql = fs.readFileSync(path.join(root, "..", "migrations", name), "utf8");
    await client.query(sql);
    console.log(`Applied ${name}`);

    const verify = await client.query(
        `SELECT slug, label, base_commission_percent, has_baseline FROM public.upline_role_types ORDER BY sort_order ASC`,
    );
    console.log(JSON.stringify(verify.rows, null, 2));
} finally {
    await client.end();
}
