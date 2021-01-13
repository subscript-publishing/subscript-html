set -e

if [ "$1" == "serve" ]
then
    cargo run -- serve --manifest ../school-notes-spring-2020/subscript.toml --open-browser
else
    cargo run -- compile --manifest ../school-notes-spring-2020/subscript.toml
fi


