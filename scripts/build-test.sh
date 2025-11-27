#!/usr/bin/env bash

###############################################################################
# Local Build Testing Script
# 
# This script simulates the GitHub Actions workflow locally for testing.
# Useful for validating build configuration before pushing to GitHub.
#
# Usage:
#   ./scripts/build-test.sh [--platform linux|windows|macos] [--version 2025.11.24]
#
###############################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PLATFORMS=("linux" "windows" "macos")
VERSION=""
PLATFORM="all"
DRY_RUN=false

# Functions
print_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
    -p, --platform PLATFORM    Build specific platform (linux|windows|macos|all)
                               Default: all
    -v, --version VERSION      Set version (CalVer format: YYYY.MM.DD)
                               Default: {base}+{commit-hash}
    -d, --dry-run              Show what would be done without building
    -h, --help                 Show this help message

Examples:
    $0 --platform linux                    # Build Linux only
    $0 --version 2025.11.24                # Release build
    $0 --platform macos --dry-run          # Show macOS build steps
EOF
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--platform)
            PLATFORM="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate platform
if [[ ! " ${PLATFORMS[@]} all " =~ " ${PLATFORM} " ]]; then
    print_error "Invalid platform: $PLATFORM"
    print_usage
    exit 1
fi

# Get commit hash and version
SHORT_HASH=$(git rev-parse --short HEAD)
if [ -z "$VERSION" ]; then
    BASE_VERSION=$(jq -r '.version' package.json)
    VERSION="${BASE_VERSION}+${SHORT_HASH}"
fi

print_info "Build Configuration"
echo "  Platform(s): $PLATFORM"
echo "  Version: $VERSION"
echo "  Commit: $SHORT_HASH"
echo "  Dry Run: $DRY_RUN"
echo ""

# Function to build platform
build_platform() {
    local platform=$1
    local target=$2
    
    print_info "Building for $platform ($target)..."
    
    if [ "$DRY_RUN" = true ]; then
        echo "  $ jq '.version = \"$VERSION\"' package.json > package.json.tmp"
        echo "  $ mv package.json.tmp package.json"
        echo "  $ jq '.version = \"$VERSION\" | .bundle.macOS.bundleVersion = \"$SHORT_HASH\"' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp"
        echo "  $ mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json"
        echo "  $ bun ci"
        echo "  $ bun tauri:build -- --target $target"
        return
    fi
    
    # Update versions
    jq ".version = \"$VERSION\"" package.json > package.json.tmp
    mv package.json.tmp package.json
    
    jq ".version = \"$VERSION\" | .bundle.macOS.bundleVersion = \"$SHORT_HASH\"" src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp
    mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
    
    # Install dependencies (only once)
    if [ ! -d "node_modules" ]; then
        print_info "Installing dependencies..."
        bun ci
    fi
    
    # Generate frontend
    print_info "Building frontend and Tauri app for $platform..."
    bun tauri:build -- --target "$target"
    
    print_success "Build completed for $platform"
}

# Validate prerequisites
print_info "Checking prerequisites..."

if ! command -v git &> /dev/null; then
    print_error "git is not installed"
    exit 1
fi
print_success "git found"

if ! command -v jq &> /dev/null; then
    print_error "jq is not installed"
    exit 1
fi
print_success "jq found"

if ! command -v npm &> /dev/null; then
    print_error "npm is not installed"
    exit 1
fi
print_success "npm found"

if ! command -v cargo &> /dev/null; then
    print_error "cargo/rust is not installed"
    exit 1
fi
print_success "cargo found"

echo ""

# Build selected platforms
if [ "$PLATFORM" = "all" ]; then
    build_platform "linux" "x86_64-unknown-linux-gnu"
    build_platform "windows" "x86_64-pc-windows-msvc"
    build_platform "macos" "aarch64-apple-darwin"
    build_platform "macos" "x86_64-apple-darwin"
elif [ "$PLATFORM" = "linux" ]; then
    build_platform "linux" "x86_64-unknown-linux-gnu"
elif [ "$PLATFORM" = "windows" ]; then
    build_platform "windows" "x86_64-pc-windows-msvc"
elif [ "$PLATFORM" = "macos" ]; then
    build_platform "macos" "aarch64-apple-darwin"
    build_platform "macos" "x86_64-apple-darwin"
fi

echo ""
print_success "Build testing completed!"
print_info "Artifacts location:"
echo "  src-tauri/target/*/release/bundle/"
