#!/bin/bash
# Setup Cron Job for XStream Integration Auto-Continuation
# Run this script to install the 1am automation

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUTO_SCRIPT="$SCRIPT_DIR/auto_continue_xstream.sh"

echo "🚀 Setting up XStream integration auto-continuation cron job..."

# Verify auto script exists and is executable
if [[ ! -x "$AUTO_SCRIPT" ]]; then
    echo "❌ Auto script not found or not executable: $AUTO_SCRIPT"
    exit 1
fi

# Create cron job entry
CRON_ENTRY="0 1 * * * $AUTO_SCRIPT >> /tmp/xstream_cron.log 2>&1"

# Check if cron job already exists
if crontab -l 2>/dev/null | grep -q "auto_continue_xstream.sh"; then
    echo "⚠️  Cron job already exists, updating..."
    # Remove existing entry and add new one
    (crontab -l 2>/dev/null | grep -v "auto_continue_xstream.sh"; echo "$CRON_ENTRY") | crontab -
else
    echo "➕ Adding new cron job..."
    # Add to existing crontab
    (crontab -l 2>/dev/null || true; echo "$CRON_ENTRY") | crontab -
fi

echo "✅ Cron job installed successfully!"
echo "📋 Current crontab:"
crontab -l | grep -A1 -B1 "auto_continue_xstream.sh" || echo "No related entries found"

echo ""
echo "🕐 The script will run daily at 1:00 AM"
echo "📝 Logs will be saved to /tmp/xstream_cron.log"
echo "🛑 To remove: crontab -e and delete the line containing 'auto_continue_xstream.sh'"

# Create initial log file
touch /tmp/xstream_cron.log
echo "$(date): XStream auto-continuation cron job installed" >> /tmp/xstream_cron.log

echo "🎉 Setup complete!"