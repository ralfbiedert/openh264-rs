#
# Updates OpenH264 hashes
#
PROJECT_ROOT="$( cd "$(dirname "$0")/.." ; pwd -P )"

VERSION=2.4.0
TARGET_FILE="$PROJECT_ROOT/openh264-sys2/src/blobs/hashes.txt"

LIBRARIES=(
  http://ciscobinary.openh264.org/libopenh264-$VERSION-android-arm.7.so.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-android-arm64.7.so.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-android-x64.7.so.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-android-x86.7.so.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-ios.a.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-linux32.7.so.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-linux64.7.so.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-linux-arm.7.so.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-linux-arm64.7.so.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-mac-arm64.dylib.bz2
  http://ciscobinary.openh264.org/libopenh264-$VERSION-mac-x64.dylib.bz2
  http://ciscobinary.openh264.org/openh264-$VERSION-win32.dll.bz2
  http://ciscobinary.openh264.org/openh264-$VERSION-win64.dll.bz2
)

rm "$TARGET_FILE"

mkdir "$PROJECT_ROOT/target"
pushd "$PROJECT_ROOT/target"

echo "Downloading OpenH264 blobs and computing their SHAs ..."

for url in "${LIBRARIES[@]}"
do
    echo "... ${url}"
    file_bz2=$(basename "$url")
    file=$(basename "$file_bz2" .bz2)
    curl -so "$file_bz2" "$url"
    bunzip2 -qf "$file_bz2"
    sha256sum "$file" >> "$TARGET_FILE"
done

echo "Updated $TARGET_FILE"

popd
