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

const migrationPath = path.join(root, "..", "migrations", "20260811_commission_period_status_paid_at.sql");
const sql = fs.readFileSync(migrationPath, "utf8");

const client = new pg.Client({
    connectionString: databaseUrl.replace(/\?.*$/, ""),
    ssl: { rejectUnauthorized: false },
});

await client.connect();
try {
    await client.query(sql);

    const verify = await client.query(`
      SELECT column_name, data_type, column_default
      FROM information_schema.columns
      WHERE table_schema = 'public'
        AND table_name = 'commission_period_status'
        AND column_name = 'paid_at'
    `);

    console.log("Migration applied.");
    console.log(JSON.stringify(verify.rows, null, 2));
} finally {
    await client.end();
}
