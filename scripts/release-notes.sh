set -e

rm -rf ../school-notes-spring-2020/docs
rm -rf ../school-notes-spring-2020/output-release
rm -rf ../school-notes-spring-2020/output

cargo run -- compile \
    --manifest ../school-notes-spring-2020/subscript.toml \
    --output-dir ../school-notes-spring-2020/output-release \
    --base-url https://colbyn.github.io/school-notes-spring-2020/

rm -rf ../school-notes-spring-2020/docs
mv ../school-notes-spring-2020/output-release ../school-notes-spring-2020/docs

cd ../school-notes-spring-2020

git add docs
git commit -m "update site"
git push
