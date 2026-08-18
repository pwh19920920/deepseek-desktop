#!/usr/bin/env bash
set -euo pipefail

echo "=== DeepSeek Harness Desktop - Clean ==="

# 清理 Rust 构建产物
echo "[1/4] Cleaning Rust build artifacts..."
rm -rf apps/desktop/src-tauri/target
rm -rf apps/desktop/src-tauri/.cache
echo "  ✅ target/ 已清理"

# 清理 Tauri 生成文件
echo "[2/4] Cleaning Tauri generated files..."
rm -rf apps/desktop/src-tauri/gen
echo "  ✅ gen/ 已清理"

# 清理临时文件
echo "[3/4] Cleaning temporary files..."
rm -rf apps/desktop/.tmp
rm -rf apps/desktop/src-tauri/.tmp
rm -rf .tmp
echo "  ✅ .tmp/ 已清理"

# 清理 Node 模块和锁文件（可选）
echo "[4/4] Cleaning Node.js artifacts (optional)..."
echo "  如需清理 node_modules，请运行："
echo "    rm -rf apps/desktop/node_modules apps/desktop/pnpm-lock.yaml"
echo ""
echo "=== Clean complete ==="
