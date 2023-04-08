#!/bin/bash

rm -rf bundler nodejs web
wasm-pack build --target bundler --out-dir bundler
wasm-pack build --target nodejs --out-dir nodejs
wasm-pack build --target web --out-dir web

echo "Checking that the NodeJS example works..."
node ../examples/nodejs

echo "Checking that the Webpack example works..."
cd ../examples/webpack
npm install
npm run build

echo "========================="
echo "Build and examples are OK"
