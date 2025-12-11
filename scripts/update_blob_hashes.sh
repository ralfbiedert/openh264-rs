#!/bin/bash
#
# Updates OpenH264 hashes
#

set -euo pipefail

PROJECT_ROOT="$( cd "$(dirname "$0")/.." ; pwd -P )"

VERSION=2.6.0 # <-- Update to latest
ABIVERSION=8
TARGET_FILE="$PROJECT_ROOT/openh264-sys2/src/blobs/hashes.txt"
CISCO_ROOT=http://ciscobinary.openh264.org
REFERENCE_FILE="openh264-$VERSION-win64.dll"
REFERENCE_FILE_PATH="$PROJECT_ROOT/openh264-sys2/tests/reference/"
REFERENCE_FILE_SPEC="$PROJECT_ROOT/openh264-sys2/tests/reference/reference.txt"
LIBRARIES=(
  $CISCO_ROOT/libopenh264-$VERSION-android-arm.$ABIVERSION.so.bz2
  $CISCO_ROOT/libopenh264-$VERSION-android-arm64.$ABIVERSION.so.bz2
  $CISCO_ROOT/libopenh264-$VERSION-android-x64.$ABIVERSION.so.bz2
  $CISCO_ROOT/libopenh264-$VERSION-android-x86.$ABIVERSION.so.bz2
  $CISCO_ROOT/libopenh264-$VERSION-ios.a.bz2
  $CISCO_ROOT/libopenh264-$VERSION-linux32.$ABIVERSION.so.bz2
  $CISCO_ROOT/libopenh264-$VERSION-linux64.$ABIVERSION.so.bz2
  $CISCO_ROOT/libopenh264-$VERSION-linux-arm.$ABIVERSION.so.bz2
  $CISCO_ROOT/libopenh264-$VERSION-linux-arm64.$ABIVERSION.so.bz2
  $CISCO_ROOT/libopenh264-$VERSION-mac-arm64.dylib.bz2
  $CISCO_ROOT/libopenh264-$VERSION-mac-x64.dylib.bz2
  $CISCO_ROOT/openh264-$VERSION-win32.dll.bz2
  $CISCO_ROOT/openh264-$VERSION-win64.dll.bz2
)

rm -f "$TARGET_FILE"

mkdir -p "$PROJECT_ROOT/target"
pushd "$PROJECT_ROOT/target"

echo "Downloading OpenH264 blobs and computing their SHAs ..."

for url in "${LIBRARIES[@]}"
do
    file_bz2=$(basename "$url")
    file=$(basename "$file_bz2" .bz2)

    echo "... ${url}"
    
    curl -f -so "$file_bz2" "$url" || (echo "FAILED TO RETRIEVE FILE" && false)
    bunzip2 -qf "$file_bz2"
    sha256sum "$file" >> "$TARGET_FILE"

    # Make sure the reference file for our unit tests is up to date
    if [ "$file" == "$REFERENCE_FILE" ]; then
        cp "$file" "$REFERENCE_FILE_PATH"
        echo -n "$REFERENCE_FILE" > "$REFERENCE_FILE_SPEC"
    fi
done

echo "Updated $TARGET_FILE"

echo "Downloading reference file for unit tests ..."


popd
