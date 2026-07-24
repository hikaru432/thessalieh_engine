#!/usr/bin/env node

// Calls POST /auth/register directly, since the frontend no longer has a
// signup UI. The endpoint requires an Ed25519-signed envelope
// { payload, pubkey, signature } (see src/api/verified.rs) — the same
// scheme the frontend's signedPost() used, but with a throwaway keypair
// generated fresh on every run instead of one persisted in IndexedDB.
//
// All registrations create users.role = "User". The optional -t access_token
// validates the invite only; it does not grant Admin. Assign Lead Broker,
// Titling Officer, or Agent via Admin → Marketing → Broker & agent roster;
// that syncs users.role to match the roster role (reverts to User on remove).
//
// Usage:
//   node register-cli.mjs -u <username> -p <password> [-t <access_token>] [-a <api_base_url>]
//
// Requires Node 20+ (uses the built-in Web Crypto Ed25519 support).

import { webcrypto } from "node:crypto";

function arg(flag, fallback) {
    const i = process.argv.indexOf(flag);
    return i !== -1 ? process.argv[i + 1] : fallback;
}

const username = arg("-u");
const password = arg("-p");
const accessToken = arg("-t", "");
const apiBase = arg(
    "-a",
    process.env.VITE_API_URL || "http://localhost:8080"
);

if (!username || !password) {
    console.error(
        "Usage: node register-cli.mjs -u <username> -p <password> [-t <access_token>] [-a <api_base_url>]"
    );
    process.exit(1);
}

const b64 = (bytes) => Buffer.from(bytes).toString("base64");

async function main() {
    // Generate a temporary Ed25519 keypair
    const { publicKey, privateKey } = await webcrypto.subtle.generateKey(
        { name: "Ed25519" },
        true,
        ["sign", "verify"]
    );

    // Create the payload
    const payloadObj = {
        username,
        password,
        access_token: accessToken,
        nonce: b64(webcrypto.getRandomValues(new Uint8Array(16))),
        ingress_expiry: Date.now() + 2 * 60 * 1000, // 2 minutes
    };

    const encoder = new TextEncoder();
    const msg = encoder.encode(JSON.stringify(payloadObj));

    // Export public key
    const pubkeyRaw = await webcrypto.subtle.exportKey("raw", publicKey);

    // Sign payload
    const signature = await webcrypto.subtle.sign(
        "Ed25519",
        privateKey,
        msg
    );

    // Build request envelope
    const envelope = {
        payload: b64(msg),
        pubkey: b64(pubkeyRaw),
        signature: b64(signature),
    };

    // Send request
    const res = await fetch(`${apiBase}/auth/register`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(envelope),
    });

    const text = await res.text();

    console.log(`HTTP ${res.status}`);

    // Print Set-Cookie header if available
    if (typeof res.headers.getSetCookie === "function") {
        const cookies = res.headers.getSetCookie();
        if (cookies.length) {
            console.log("Set-Cookie:", cookies.join(" | "));
        }
    } else {
        const cookie = res.headers.get("set-cookie");
        if (cookie) {
            console.log("Set-Cookie:", cookie);
        }
    }

    console.log(text);

    process.exit(res.ok ? 0 : 1);
}

main().catch((err) => {
    console.error("Request failed:", err?.message ?? err);
    process.exit(1);
});