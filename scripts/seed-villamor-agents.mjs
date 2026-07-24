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
      SELECT r.id, u.username, r.role, r.commission_rate, r.broker_id
      FROM public.roster r
      JOIN public.users u ON u.id = r.user_id
      WHERE r.status = 'Active'
      ORDER BY u.username
    `);

    const byUsername = new Map(
        roster.rows.map((row) => [String(row.username).toLowerCase(), row]),
    );

    const gladez = byUsername.get("gladez");
    const rosa = byUsername.get("rosa");
    const hikaru = byUsername.get("hikaru");

    if (!gladez || !rosa || !hikaru) {
        console.error("Missing roster entries:", {
            gladez: Boolean(gladez),
            rosa: Boolean(rosa),
            hikaru: Boolean(hikaru),
        });
        process.exit(1);
    }

    const agentsJson = [
        {
            id: gladez.id,
            name: gladez.username,
            nickname: gladez.username.split(/\s+/)[0] || gladez.username,
            role: "lead-broker",
            parentId: null,
            sharePercent: 0,
        },
        {
            id: rosa.id,
            name: rosa.username,
            nickname: rosa.username.split(/\s+/)[0] || rosa.username,
            role: "titling-officer",
            parentId: null,
            sharePercent: 0,
        },
        {
            id: hikaru.id,
            name: hikaru.username,
            nickname: hikaru.username.split(/\s+/)[0] || hikaru.username,
            role: "downline",
            parentId: gladez.id,
            sharePercent: Number(hikaru.commission_rate) || 0,
        },
    ];

    const result = await client.query(
        `UPDATE public.projects
            SET agents_json = $1::jsonb, updated_at = EXTRACT(EPOCH FROM NOW())::bigint
          WHERE name = 'Villamor Village'
      RETURNING id, name, agents_json`,
        [JSON.stringify(agentsJson)],
    );

    if (result.rowCount === 0) {
        console.error("Villamor Village project not found");
        process.exit(1);
    }

    const projects = await client.query(
        `SELECT name, jsonb_array_length(agents_json) AS agent_count
         FROM public.projects
         ORDER BY name`,
    );

    console.log("Seeded Villamor Village agents_json:");
    console.log(JSON.stringify(result.rows[0], null, 2));
    console.log("All projects:");
    console.log(JSON.stringify(projects.rows, null, 2));
} finally {
    await client.end();
}
