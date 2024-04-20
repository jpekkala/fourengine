#!/bin/bash

echo "======================="
echo "Copying opening books  "
echo "======================="

# npm does not follow symlinks so have to copy the folder
(set -x; rm -rf ./books; cp -r ../books ./books)

echo
echo "======================="
echo "Building with wasm-pack"
echo "======================="
echo
rm -rf bundler nodejs web
wasm-pack build --target bundler --out-dir bundler
wasm-pack build --target nodejs --out-dir nodejs
wasm-pack build --target web --out-dir web

# wasm-pack adds .gitignore with a single wildcard which then prevents npm pack from including these folders
rm bundler/.gitignore nodejs/.gitignore web/.gitignore

echo
echo "======================================"
echo "Checking that the NodeJS example works"
echo "======================================"
echo
(cd ../examples/nodejs; node index.js)

echo
echo "======================================="
echo "Checking that the Webpack example works"
echo "======================================="
echo
(cd ../examples/webpack; npm install; npm run build)

echo
echo "========================="
echo "Build and examples are OK"
echo "========================="
echo
npm pack --dry-run

echo
echo "==========================================="
echo
echo "The output above is from npm-pack --dry-run"
echo
echo 'Type "npm pack" to create a tarball or "npm publish" to publicly publish'