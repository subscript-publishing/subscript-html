set -e

watchexec --exts purs --ignore 'output/ ./index.js generated-docs/ .spago/' -- 'spago bundle-app && echo "\n==reloaded==\n"'
