#!/bin/bash
#
# Updates the hashes for one OpenH264 version.
# Usage: update_blob_hashes.sh VERSION [ABI_VERSION]
#

set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
    echo "Usage: $0 VERSION [ABI_VERSION]" >&2
    exit 1
fi

VERSION="$1"
ABIVERSION="${2:-8}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ || ! "$ABIVERSION" =~ ^[0-9]+$ ]]; then
    echo "Expected a version such as 2.7.0 and a numeric ABI version." >&2
    exit 1
fi

PROJECT_ROOT="$( cd "$(dirname "$0")/.." ; pwd -P )"
TARGET_FILE="$PROJECT_ROOT/openh264-sys2/src/blobs/hashes-$VERSION.txt"
CISCO_ROOT=https://ciscobinary.openh264.org
DOWNLOAD_DIR="$PROJECT_ROOT/target/openh264-blobs"
TEMP_HASHES=$(mktemp)
trap 'rm -f "$TEMP_HASHES"' EXIT

mkdir -p "$DOWNLOAD_DIR"
pushd "$DOWNLOAD_DIR" > /dev/null

echo "Downloading OpenH264 blobs and computing their SHAs ..."

LIBRARIES=(
  "libopenh264-$VERSION-android-arm.$ABIVERSION.so"
  "libopenh264-$VERSION-android-arm64.$ABIVERSION.so"
  "libopenh264-$VERSION-android-x64.$ABIVERSION.so"
  "libopenh264-$VERSION-android-x86.$ABIVERSION.so"
  "libopenh264-$VERSION-ios.a"
  "libopenh264-$VERSION-linux32.$ABIVERSION.so"
  "libopenh264-$VERSION-linux64.$ABIVERSION.so"
  "libopenh264-$VERSION-linux-arm.$ABIVERSION.so"
  "libopenh264-$VERSION-linux-arm64.$ABIVERSION.so"
  "libopenh264-$VERSION-mac-arm64.dylib"
  "libopenh264-$VERSION-mac-x64.dylib"
  "openh264-$VERSION-win32.dll"
  "openh264-$VERSION-win64.dll"
  "openh264-$VERSION-win-arm64.dll"
)

for file in "${LIBRARIES[@]}"; do
    url="$CISCO_ROOT/$file.bz2"
    echo "... ${url}"

    curl --fail --location --silent --show-error --retry 3 --output "$file.bz2" "$url"
    bunzip2 -qf "$file.bz2"
    sha256sum "$file" >> "$TEMP_HASHES"
done

# Only replace this version's hashes after all downloads succeed.
cp "$TEMP_HASHES" "$TARGET_FILE"

echo "Updated $TARGET_FILE"

popd > /dev/null
