#!/bin/bash
set -euo pipefail

# Script to incrementally update CHANGELOG.md with unreleased changes
# Usage: ./scripts/update_changelog.sh

if ! command -v git-cliff &> /dev/null; then
    echo "❌ git-cliff not found. Install it with: cargo install git-cliff"
    exit 1
fi

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not in a git repository"
    exit 1
fi

# Check if CHANGELOG.md exists
if [[ ! -f "CHANGELOG.md" ]]; then
    echo "❌ CHANGELOG.md not found"
    exit 1
fi

echo "📝 Generating unreleased changes..."

# Generate unreleased changes to a temporary file
TEMP_CONTENT=$(mktemp)
git cliff --unreleased --strip header > "$TEMP_CONTENT"

# Check if there are any unreleased changes
if [[ $(wc -l < "$TEMP_CONTENT") -le 1 ]]; then
    echo "✅ No unreleased changes found"
    rm "$TEMP_CONTENT"
    exit 0
fi

echo "🔄 Updating CHANGELOG.md..."

# Create a backup
cp CHANGELOG.md CHANGELOG.md.bak

# Check if there's already an unreleased section
if grep -q "^## \[Unreleased\]" CHANGELOG.md; then
    # Replace the existing unreleased section
    awk '
    /^## \[Unreleased\]/ {
        while (getline line < "'"$TEMP_CONTENT"'") {
            print line
        }
        # Skip the original unreleased content until next section
        while (getline > 0) {
            if (/^## \[.*\]/ && !/^## \[Unreleased\]/) {
                print
                break
            }
        }
        next
    }
    { print }
    ' CHANGELOG.md > CHANGELOG.md.tmp
else
    # Prepend unreleased section at the top
    cat "$TEMP_CONTENT" <(echo "") CHANGELOG.md > CHANGELOG.md.tmp
fi

# Replace the original file
mv CHANGELOG.md.tmp CHANGELOG.md
rm "$TEMP_CONTENT"

echo "✅ CHANGELOG.md updated successfully"
echo ""
echo "📋 Unreleased changes added:"
head -20 "$TEMP_CONTENT" 2>/dev/null || echo "(Content already applied)"
