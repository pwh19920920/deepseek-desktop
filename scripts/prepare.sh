#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

echo "=== DeepSeek Harness Desktop - Prepare ==="

# Step 1: Fetch Node.js
echo "[1/2] Fetching Node.js..."
bash "${SCRIPT_DIR}/fetch-node.sh"

# Step 2: Bundle dsh dependencies
echo "[2/2] Bundling dsh dependencies..."
bash "${SCRIPT_DIR}/bundle-dsh.sh"

echo "=== Prepare complete ==="
