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
});

await client.connect();
try {
    const migrations = [
        "20260727b_commission_split_schedule.sql",
        "20260727c_contract_amort_start_date.sql",
    ];
    for (const name of migrations) {
        const sql = fs.readFileSync(path.join(root, "..", "migrations", name), "utf8");
        await client.query(sql);
        console.log(`Applied ${name}`);
    }

    const verify = await client.query(
        "SELECT effective_date, split_months FROM public.commission_split_schedule ORDER BY effective_date ASC",
    );
    console.log(JSON.stringify(verify.rows, null, 2));
} finally {
    await client.end();
}
