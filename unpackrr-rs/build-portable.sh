#!/usr/bin/env bash
# Build script for creating portable Unpackrr distribution
# Usage: ./build-portable.sh [version]
# Example: ./build-portable.sh 0.1.0

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Version (default to 0.1.0 if not provided)
VERSION="${1:-0.1.0}"

# Directories
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"
BUILD_DIR="${DIST_DIR}/unpackrr-${VERSION}"

# Binary name based on platform
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    BINARY_NAME="unpackrr.exe"
    PLATFORM="windows"
else
    BINARY_NAME="unpackrr"
    PLATFORM="linux"
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Unpackrr Portable Build Script${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}Version:${NC} ${VERSION}"
echo -e "${GREEN}Platform:${NC} ${PLATFORM}"
echo -e "${GREEN}Build Directory:${NC} ${BUILD_DIR}"
echo ""

# Step 1: Clean previous builds
echo -e "${YELLOW}[1/7]${NC} Cleaning previous builds..."
rm -rf "${DIST_DIR}"
mkdir -p "${BUILD_DIR}"

# Step 2: Build release binary
echo -e "${YELLOW}[2/7]${NC} Building release binary..."
cd "${PROJECT_ROOT}"
cargo build --release

if [ ! -f "target/release/${BINARY_NAME}" ]; then
    echo -e "${RED}ERROR:${NC} Release binary not found at target/release/${BINARY_NAME}"
    exit 1
fi

# Step 3: Copy binary
echo -e "${YELLOW}[3/7]${NC} Copying binary..."
cp "target/release/${BINARY_NAME}" "${BUILD_DIR}/"

# Step 4: Copy documentation
echo -e "${YELLOW}[4/7]${NC} Copying documentation..."
cp "${PROJECT_ROOT}/README.md" "${BUILD_DIR}/"
cp "${PROJECT_ROOT}/THIRD_PARTY_LICENSES.md" "${BUILD_DIR}/"

# Copy main license (GPL-3.0)
if [ -f "${PROJECT_ROOT}/../LICENSE" ]; then
    cp "${PROJECT_ROOT}/../LICENSE" "${BUILD_DIR}/LICENSE"
elif [ -f "${PROJECT_ROOT}/LICENSE" ]; then
    cp "${PROJECT_ROOT}/LICENSE" "${BUILD_DIR}/LICENSE"
else
    echo -e "${YELLOW}WARNING:${NC} LICENSE file not found, creating placeholder..."
    cat > "${BUILD_DIR}/LICENSE" << 'EOF'
Unpackrr - BA2 Batch Unpacker (Rust Edition)
Copyright (C) 2024-2025 evildarkarchon

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
EOF
fi

# Step 5: Check for BSArch.exe (Windows only)
echo -e "${YELLOW}[5/7]${NC} Checking for BSArch.exe..."
if [ -f "${PROJECT_ROOT}/BSArch.exe" ]; then
    cp "${PROJECT_ROOT}/BSArch.exe" "${BUILD_DIR}/"
    echo -e "${GREEN}✓${NC} BSArch.exe included"
elif [ -f "${PROJECT_ROOT}/../BSArch.exe" ]; then
    cp "${PROJECT_ROOT}/../BSArch.exe" "${BUILD_DIR}/"
    echo -e "${GREEN}✓${NC} BSArch.exe included"
else
    echo -e "${YELLOW}WARNING:${NC} BSArch.exe not found in project directory"
    echo -e "${YELLOW}         Please download from https://github.com/TES5Edit/TES5Edit${NC}"
    echo -e "${YELLOW}         and place in: ${BUILD_DIR}/${NC}"
    # Create a README placeholder
    cat > "${BUILD_DIR}/BSARCH_NEEDED.txt" << 'EOF'
BSArch.exe is required for Unpackrr to function.

Please download BSArch.exe from:
https://github.com/TES5Edit/TES5Edit/releases

Place BSArch.exe in the same directory as unpackrr.exe

BSArch.exe is licensed under MPL-2.0 (Mozilla Public License 2.0).
See THIRD_PARTY_LICENSES.md for details.
EOF
fi

# Step 6: Create version info file
echo -e "${YELLOW}[6/7]${NC} Creating version info..."
cat > "${BUILD_DIR}/VERSION.txt" << EOF
Unpackrr Version ${VERSION}
Build Date: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
Platform: ${PLATFORM}
Rust Version: $(rustc --version)
EOF

# Step 7: Create distribution archive
echo -e "${YELLOW}[7/7]${NC} Creating distribution archive..."
cd "${DIST_DIR}"
ARCHIVE_NAME="unpackrr-${VERSION}-${PLATFORM}.tar.gz"

if command -v tar &> /dev/null; then
    tar -czf "${ARCHIVE_NAME}" "unpackrr-${VERSION}"
    echo -e "${GREEN}✓${NC} Created: ${DIST_DIR}/${ARCHIVE_NAME}"
else
    echo -e "${YELLOW}WARNING:${NC} tar not available, skipping archive creation"
fi

# Summary
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Build Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Distribution directory:${NC} ${BUILD_DIR}"
if [ -f "${DIST_DIR}/${ARCHIVE_NAME}" ]; then
    echo -e "${GREEN}Archive:${NC} ${DIST_DIR}/${ARCHIVE_NAME}"
fi

# Calculate sizes
BINARY_SIZE=$(du -h "${BUILD_DIR}/${BINARY_NAME}" | cut -f1)
echo -e "${GREEN}Binary size:${NC} ${BINARY_SIZE}"

if [ -f "${BUILD_DIR}/BSArch.exe" ]; then
    BSARCH_SIZE=$(du -h "${BUILD_DIR}/BSArch.exe" | cut -f1)
    echo -e "${GREEN}BSArch.exe size:${NC} ${BSARCH_SIZE}"
fi

echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Test the distribution: cd ${BUILD_DIR} && ./${BINARY_NAME}"
if [ ! -f "${BUILD_DIR}/BSArch.exe" ]; then
    echo -e "  2. ${YELLOW}Download and add BSArch.exe to ${BUILD_DIR}${NC}"
fi
echo -e "  3. Create release on GitHub with the archive"
echo ""
