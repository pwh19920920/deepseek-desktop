#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

echo "=== DeepSeek Harness Desktop - Build ==="

# Step 1: Prepare (fetch node + bundle dsh)
echo "[1/3] Preparing..."
bash "${SCRIPT_DIR}/prepare.sh"

# Step 2: Build frontend
echo "[2/3] Building frontend..."
cd apps/desktop
pnpm build

# Step 3: Build Tauri app
echo "[3/3] Building Tauri app..."
pnpm tauri build

echo "=== Build complete ==="
