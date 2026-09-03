# ============================================
# Supabase Database Backup
# ============================================

$ErrorActionPreference = "Stop"

# Project location
$ProjectDir = "D:\thessalieh_engine"

# Backup directory
$BackupRoot = Join-Path $ProjectDir "backups"

# Create a dated folder
$Date = Get-Date -Format "yyyy-MM-dd"
$BackupDir = Join-Path $BackupRoot $Date

# Create backup folder if it doesn't exist
New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null

Write-Host "Starting Supabase backup..."
Write-Host "Backup location: $BackupDir"

# Check database URL
if (-not $env:SUPABASE_DB_URL) {
    Write-Error "SUPABASE_DB_URL is not set."
    exit 1
}

# Backup schema
Write-Host "Backing up schema..."

npx supabase db dump `
    --db-url $env:SUPABASE_DB_URL `
    -f "$BackupDir\schema.sql"

# Backup data
Write-Host "Backing up data..."

npx supabase db dump `
    --db-url $env:SUPABASE_DB_URL `
    -f "$BackupDir\data.sql" `
    --use-copy `
    --data-only

Write-Host ""
Write-Host "============================================"
Write-Host "Backup completed successfully!"
Write-Host "Location: $BackupDir"
Write-Host "============================================"