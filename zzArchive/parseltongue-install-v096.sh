#!/bin/bash

# Parseltongue v0.9.6 Install Script
# Test Exclusion (90% token reduction) + Single Binary Architecture (80% disk reduction)

set -e

echo "🐍 Parseltongue v0.9.6 Installer"
echo "=================================="
echo "Features: Test Exclusion (90% token savings) + Single Binary (80% disk savings)"
echo ""

# Check if git repo exists (required for installation)
if [ ! -d ".git" ]; then
    echo "❌ Error: Must be run from a git repository root"
    echo "   This is required for proper ISG analysis functionality"
    exit 1
fi

# Detect platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $PLATFORM in
    darwin)
        if [ "$ARCH" = "arm64" ]; then
            BINARY_URL="https://github.com/that-in-rust/parseltongue/releases/download/v0.9.6/parseltongue"
        else
            echo "❌ Error: macOS x86_64 not supported in this release"
            exit 1
        fi
        ;;
    linux)
        if [ "$ARCH" = "x86_64" ]; then
            echo "❌ Error: Linux x86_64 not yet available for v0.9.6"
            echo "   Please use v0.9.2 or build from source"
            exit 1
        else
            echo "❌ Error: Linux ARM64 not supported in this release"
            exit 1
        fi
        ;;
    *)
        echo "❌ Error: Platform $PLATFORM not supported"
        exit 1
        ;;
esac

# Download binary as 'parseltongue'
echo "📥 Downloading Parseltongue v0.9.6 for $PLATFORM-$ARCH..."
curl -L -o parseltongue "$BINARY_URL"

# Make executable
chmod +x parseltongue

# Create .claude directories for agents
mkdir -p .claude/.parseltongue
mkdir -p .claude/agents

# Download agent files
echo "📥 Installing ISG Explorer agent..."
curl -L https://raw.githubusercontent.com/that-in-rust/parseltongue/main/.claude/agents/parseltongue-ultrathink-isg-explorer.md \
  -o .claude/agents/parseltongue-ultrathink-isg-explorer.md

# Download documentation
echo "📥 Installing documentation..."
curl -L https://raw.githubusercontent.com/that-in-rust/parseltongue/main/README.md \
  -o .claude/.parseltongue/README.md

# Verify installation
echo ""
echo "✅ Installation complete!"
echo ""
echo "🎯 v0.9.6 Features:"
echo "   • Test Exclusion: CODE entities only (90% token reduction)"
echo "   • Single Binary: 49MB unified binary (80% disk reduction)"
echo "   • All pt01-pt07 commands accessible via one binary"
echo "   • Zero breaking changes - all existing commands work"
echo ""
echo "🚀 Quick start:"
echo "   ./parseltongue pt01-folder-to-cozodb-streamer . --db rocksdb:mycode.db"
echo "   # Tests automatically excluded - only CODE entities ingested"
echo ""
echo "   ./parseltongue pt02-level00 --where-clause \"ALL\" --output edges.json --db rocksdb:mycode.db"
echo "   # Clean dependency graph without test noise"
echo ""
echo "🤖 Agent usage:"
echo "   Restart Claude Code, then use: @parseltongue-ultrathink-isg-explorer"
echo ""
echo "📚 Documentation: .claude/.parseltongue/README.md"
