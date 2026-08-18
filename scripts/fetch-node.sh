#!/usr/bin/env bash
set -euo pipefail

# Detect platform and architecture
PLATFORM=""
ARCH=""
EXT=""
ARCHIVE=""

case "${OSTYPE}" in
  darwin*)
    PLATFORM="darwin"
    case $(uname -m) in
      arm64) ARCH="arm64" ;;
      x86_64) ARCH="x64" ;;
      *) echo "Unsupported macOS architecture: $(uname -m)" >&2; exit 1 ;;
    esac
    EXT="tar.gz"
    ;;
  linux*)
    PLATFORM="linux"
    ARCH="x64"
    EXT="tar.gz"
    ;;
  msys*|cygwin*|mingw*)
    PLATFORM="win"
    ARCH="x64"
    EXT="zip"
    ;;
  *)
    echo "Unsupported platform: ${OSTYPE}" >&2
    exit 1
    ;;
esac

# Node.js version — lock to a specific release for reproducible builds
# v24.9.0 is the current stable as of Aug 2026; update the version number when
# a new minor/patch lands. Do NOT use the "latest" redirect — pin explicitly.
NODE_VERSION="v24.9.0"
NODE_DIST_URL="https://nodejs.org/dist/${NODE_VERSION}"

# Filename on the CDN
if [[ "${PLATFORM}" == "win" ]]; then
  ARCHIVE="node-${NODE_VERSION}-${PLATFORM}-${ARCH}.zip"
else
  ARCHIVE="node-${NODE_VERSION}-${PLATFORM}-${ARCH}.${EXT}"
fi

# Resolve absolute paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BINARIES_DIR="${PROJECT_ROOT}/binaries"

# Determine the Tauri target triple for the binary name.
# In dev mode, use the current host triple (rustc -vV).
# This matches Tauri 2's externalBin naming convention:
#   binary-name-{target-triple}  (e.g. node-aarch64-apple-darwin)
TARGET_TRIPLE=$(rustc -vV 2>/dev/null | grep '^host:' | awk '{print $2}' || echo "")

if [[ -z "${TARGET_TRIPLE}" ]]; then
  echo "[fetch-node] warning: could not determine target triple, using 'node' as fallback"
  BIN_NAME="node"
elif [[ "${PLATFORM}" == "win" ]]; then
  BIN_NAME="node-${TARGET_TRIPLE}.exe"
else
  BIN_NAME="node-${TARGET_TRIPLE}"
fi

# Download directory (avoid re-downloading if already present)
DOWNLOAD_DIR="${PROJECT_ROOT}/.tmp/node-download"
mkdir -p "${DOWNLOAD_DIR}"

echo "[fetch-node] target: ${NODE_VERSION} for ${PLATFORM}-${ARCH}"
echo "[fetch-node] archive: ${ARCHIVE}"
echo "[fetch-node] output binary name: ${BIN_NAME}"

# Check if the target binary already exists
if [[ -f "${BINARIES_DIR}/${BIN_NAME}" ]]; then
  echo "[fetch-node] ${BIN_NAME} already present, skipping."
  exit 0
fi

# Download
ARTIFACT_URL="${NODE_DIST_URL}/${ARCHIVE}"
echo "[fetch-node] downloading ${ARTIFACT_URL}"
curl -fSL "${ARTIFACT_URL}" -o "${DOWNLOAD_DIR}/${ARCHIVE}"

# Prepare binaries/ directory
mkdir -p "${BINARIES_DIR}" "${DOWNLOAD_DIR}/extracted"

# Extract
echo "[fetch-node] extracting..."
if [[ "${EXT}" == "zip" ]]; then
  # Windows: zip contains a top-level directory named node-${version}-win-x64/
  unzip -q "${DOWNLOAD_DIR}/${ARCHIVE}" -d "${DOWNLOAD_DIR}/extracted"
  cp "${DOWNLOAD_DIR}/extracted/node-${NODE_VERSION}-win-x64/node.exe" "${BINARIES_DIR}/${BIN_NAME}"
else
  # macOS/Linux: tar.gz contains a top-level directory named node-${version}-${platform}-${arch}/
  tar xzf "${DOWNLOAD_DIR}/${ARCHIVE}" -C "${DOWNLOAD_DIR}/extracted"
  cp "${DOWNLOAD_DIR}/extracted/node-${NODE_VERSION}-${PLATFORM}-${ARCH}/bin/node" "${BINARIES_DIR}/${BIN_NAME}"
fi

# Make executable (not needed for .exe on Windows)
if [[ "${PLATFORM}" != "win" ]]; then
  chmod +x "${BINARIES_DIR}/${BIN_NAME}"
fi

# Cleanup temp files
rm -rf "${DOWNLOAD_DIR}"

echo "[fetch-node] done: ${BINARIES_DIR}/${BIN_NAME}"
