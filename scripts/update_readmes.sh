#
# Update README from lib.rs.
#

PROJECT_ROOT="$( cd "$(dirname "$0")/.." ; pwd -P )" # this file

function update_readme() {
    cd "$PROJECT_ROOT"/"$1"
    cargo readme --no-license --no-title > README.md
}

update_readme "openh264"
update_readme "openh264-sys2"

cp "$PROJECT_ROOT"/openh264/README.md "$PROJECT_ROOT"