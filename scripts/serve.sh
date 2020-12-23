set -e

CARGO_FLAGS=""

if [ -z "$1" ]
then
    # NOT DEFINED
    CARGO_FLAGS+=""
else
    # OTHERWISE - DEFINED
    CARGO_FLAGS+="$1"
fi

rm -rf examples/school-notes/output
cargo run $CARGO_FLAGS -- serve --manifest examples/school-notes/subscript.toml --open-browser


