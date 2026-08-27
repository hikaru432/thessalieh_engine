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

const sql = fs.readFileSync(
    path.join(root, "..", "migrations", "20260827_roster_role_drop_check.sql"),
    "utf8",
);

const client = new pg.Client({
    connectionString: databaseUrl,
    ssl: databaseUrl.includes("supabase") ? { rejectUnauthorized: false } : undefined,
});

await client.connect();
try {
    await client.query(sql);
    const verify = await client.query(`
      SELECT conname
        FROM pg_constraint
       WHERE conrelid = 'public.roster'::regclass
         AND conname = 'roster_role_check'
    `);
    console.log("Migration applied.");
    console.log("roster_role_check still exists:", verify.rows.length > 0);
} finally {
    await client.end();
}
