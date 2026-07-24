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
    const roster = await client.query(`SELECT r.id, u.username, r.role, r.status FROM public.roster r JOIN public.users u ON u.id = r.user_id ORDER BY u.username`);
    const hikaru = await client.query(`SELECT id, username, role, updated_at FROM public.users WHERE username = 'hikaru'`);
    console.log("ROSTER COUNT:", roster.rowCount);
    console.log("ROSTER:", JSON.stringify(roster.rows, null, 2));
    console.log("HIKARU:", JSON.stringify(hikaru.rows, null, 2));
} finally {
    await client.end();
}
