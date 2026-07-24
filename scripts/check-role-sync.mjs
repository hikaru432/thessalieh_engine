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

await client.connect();
try {
    const roster = await client.query(`
      SELECT u.username, u.role AS users_role, r.role AS roster_role, r.status
      FROM public.users u
      JOIN public.roster r ON r.user_id = u.id
      ORDER BY u.username
    `);
    const mismatches = roster.rows.filter((row) => row.users_role !== row.roster_role);
    console.log("ALL ROSTER USERS:", JSON.stringify(roster.rows, null, 2));
    console.log("MISMATCHES:", JSON.stringify(mismatches, null, 2));

    const allUsers = await client.query(`
      SELECT username, role FROM public.users ORDER BY username
    `);
    console.log("ALL USERS:", JSON.stringify(allUsers.rows, null, 2));
} finally {
    await client.end();
}
