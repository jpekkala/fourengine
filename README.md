# Fourengine
A perfect Connect-4 solver

## Run in terminal
`cargo run --release`

`cargo run --release -- --variation 444444`

`cargo run --release -- --test ./test-set.c4`

## Webassembly

The engine can be run in the browser by compiling it into Webassembly

```shell
cargo install wasm-pack
# if the above command fails, check its error message because it should say what dependencies to install

cd www
wasm-pack build
npm install
npm start
```
