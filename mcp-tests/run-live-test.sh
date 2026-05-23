#!/usr/bin/env bash
# Live protocol test for rust_scraper MCP server over stdio
# Sends JSON-RPC messages and validates responses.
#
# Usage: bash mcp-tests/run-live-test.sh
set -euo pipefail

MCP_BIN="${MCP_BIN:-target/release/examples/mcp_server_stdio}"
PASS=0
FAIL=0

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Find binary
if [ ! -f "$MCP_BIN" ]; then
    echo "Building MCP server..."
    cargo build --release --example mcp_server_stdio --quiet
fi

# Helper: send JSON-RPC message and check response
test_tool() {
    local desc="$1"
    local request="$2"
    local expected="$3"

    local response
    response=$(echo "$request" | "$MCP_BIN" 2>/dev/null | head -c 5000 || true)

    if echo "$response" | grep -q "$expected"; then
        echo -e "  ${GREEN}✅ PASS${NC} $desc"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}❌ FAIL${NC} $desc"
        echo "     Expected to contain: $expected"
        echo "     Got: $(echo "$response" | head -c 300)"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== rust_scraper MCP Live Protocol Tests ==="
echo ""

# Test 1: tools/list returns registered tools
echo "📋 Tool listing:"
test_tool "tools/list returns scrape_url" \
    '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
    "scrape_url"

test_tool "tools/list returns clean_html" \
    '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
    "clean_html"

test_tool "tools/list returns validate_url" \
    '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
    "validate_url"

test_tool "tools/list returns detect_waf" \
    '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
    "detect_waf"

test_tool "tools/list returns export_file" \
    '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
    "export_file"

test_tool "tools/list returns detect_obsidian_vault" \
    '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' \
    "detect_obsidian_vault"

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ] || exit 1
