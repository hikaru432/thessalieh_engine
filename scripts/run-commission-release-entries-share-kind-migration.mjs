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
    const name = "20260807_commission_release_entries_share_kind.sql";
    const sql = fs.readFileSync(path.join(root, "..", "migrations", name), "utf8");
    await client.query(sql);
    console.log(`Applied ${name}`);

    const verify = await client.query(
        `SELECT column_name, data_type, is_nullable
           FROM information_schema.columns
          WHERE table_schema = 'public' AND table_name = 'commission_release_entries'
          ORDER BY ordinal_position`,
    );
    console.log(JSON.stringify(verify.rows, null, 2));
} finally {
    await client.end();
}
