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
    const count = await client.query(`SELECT COUNT(*)::int AS n FROM public.roster`);
    console.log("roster rows:", count.rows[0].n);

    await client.query(`
      UPDATE public.users
      SET role = 'Agent', updated_at = extract(epoch FROM now())::bigint
      WHERE username = 'hikaru' AND role != 'Admin'
    `);

    const hikaru = await client.query(`
      SELECT username, role, updated_at FROM public.users WHERE username = 'hikaru'
    `);
    console.log("hikaru after fix:", JSON.stringify(hikaru.rows, null, 2));
} finally {
    await client.end();
}
