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

const migrationPath = path.join(root, "..", "migrations", "20260724_users_role_add_agent.sql");
const sql = fs.readFileSync(migrationPath, "utf8");

const client = new pg.Client({
    connectionString: databaseUrl,
    ssl: databaseUrl.includes("supabase") ? { rejectUnauthorized: false } : undefined,
});

await client.connect();
try {
    await client.query(sql);

    const verify = await client.query(`
      SELECT u.username, u.role AS users_role, r.role AS roster_role
      FROM public.users u
      JOIN public.roster r ON r.user_id = u.id
      WHERE u.username IN ('gladez', 'rosa', 'hikaru')
      ORDER BY u.username
    `);

    console.log("Migration applied.");
    console.log(JSON.stringify(verify.rows, null, 2));
} finally {
    await client.end();
}
