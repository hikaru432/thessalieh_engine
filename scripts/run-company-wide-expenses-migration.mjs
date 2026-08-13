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

const connectionStringNoSslMode = databaseUrl.replace(/([?&])sslmode=[^&]*&?/, "$1").replace(/[?&]$/, "");

const client = new pg.Client({
    connectionString: connectionStringNoSslMode,
    ssl: databaseUrl.includes("supabase") ? { rejectUnauthorized: false } : undefined,
    connectionTimeoutMillis: 15000,
});

await client.connect();
try {
    const name = "20260816_company_wide_expenses.sql";
    const sql = fs.readFileSync(path.join(root, "..", "migrations", name), "utf8");
    await client.query(sql);
    console.log(`Applied ${name}`);

    const tables = ["expense_categories", "expenses"];
    for (const table of tables) {
        const verify = await client.query(
            `SELECT column_name, data_type, is_nullable
               FROM information_schema.columns
              WHERE table_schema = 'public' AND table_name = $1
              ORDER BY ordinal_position`,
            [table],
        );
        console.log(`${table}:`);
        console.log(JSON.stringify(verify.rows, null, 2));
    }
} finally {
    await client.end();
}
