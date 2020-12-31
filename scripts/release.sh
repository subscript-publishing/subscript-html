set -e

rm -rf docs
rm -rf examples/school-notes/output-release
rm -rf examples/school-notes/output

cargo run -- compile \
    --manifest examples/school-notes/subscript.toml \
    --output-dir output-release \
    --base-url https://colbyn.github.io/subscript/

mv examples/school-notes/output-release docs

# git add docs
# git commit -m "update site"
# git push

