#!/usr/bin/env bash
# Triggers a Render redeploy via the service's Deploy Hook, so it re-pulls
# whatever image tag it's configured to run (normally :latest).
#
# One-time setup, before this script does anything useful:
#   1. Push at least one image first (docker-build.sh, then docker-push.sh).
#   2. In the Render dashboard: New + > Web Service > Deploy an existing
#      image from a registry.
#   3. Image URL: docker.io/<DOCKERHUB_USERNAME>/thessalieh_engine:latest
#      (the Docker Hub repo must be public, or add registry credentials
#      under Render's Settings > Credentials if it's private).
#   4. Environment tab: add every var the engine needs at runtime
#      (DATABASE_URL, CLIENT_URL, DEVICE_SECRET, SUPABASE_URL,
#      SUPABASE_SECRET_KEY, SUPABASE_SERVICE_ROLE_JWT,
#      SUPABASE_STORAGE_BUCKET, WORKER_SECRET, N8N_AUTOMATION_TOKEN,
#      N8N_AUTOMATION_ADMIN_EMAIL — see .env locally for the full list
#      and current values). Render sets PORT itself; the app already
#      reads it (src/engine.rs), so don't override it.
#   5. Settings tab > Deploy Hook > copy the URL, then:
#        export RENDER_DEPLOY_HOOK_URL="https://api.render.com/deploy/srv-...?key=..."
#
# Usage (after the one-time setup above):
#   RENDER_DEPLOY_HOOK_URL=<hook-url> ./scripts/render-deploy.sh
set -euo pipefail

: "${RENDER_DEPLOY_HOOK_URL:?Set RENDER_DEPLOY_HOOK_URL to the service's Deploy Hook URL from Render (see this script's header for how to get one)}"

echo "Triggering Render deploy..."
curl -sS -X POST "${RENDER_DEPLOY_HOOK_URL}"
echo
echo "Deploy triggered — watch progress in the Render dashboard."
