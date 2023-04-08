# Fourengine
A perfect Connect-4 solver

## Quick start

Run in interactive mode:

`cargo run --release`

Solve the game:

`cargo run --release -- solve`

Solve a position:

`cargo run --release -- solve 444444`

Solve all positions from a file and verify their scores:

`cargo run --release -- test ./test-set.c4`

## Opening book
This repo does not currently include a precompiled book. It can however be generated with:

`cargo run --release -- generate-book`

In interactive mode, the engine will automatically use an opening book if it exists. The book can be explicitly disabled
with the flag --no-book:

`cargo run --release -- --no-book`

The _solve_ subcommand never uses a book even if one exists.

## Profiling

### Setup in WSL2
```shell
git clone --depth=1 https://github.com/microsoft/WSL2-Linux-Kernel.git
cd WSL2-Linux-Kernel/tools/perf

sudo apt install build-essential flex bison libssl-dev libelf-dev
make
# add WSL2-Linux-Kernel/tools/perf to $PATH
```
If building perf fails, try cloning a tag that matches your kernel version
```shell
uname -r
# 5.10.16.3-microsoft-standard-WSL2
git clone --depth=1 --branch=linux-msft-wsl-5.10.16.3 https://github.com/microsoft/WSL2-Linux-Kernel.git
```

### Profile release build
```shell
cargo build --release
perf record target/release/fourengine
perf report
```

## WebAssembly (JavaScript)

```shell
cargo install wasm-pack
# if the above command fails, check its error message because it should say what dependencies to install
````

### NodeJS example

```shell
wasm-pack build www --target nodejs --out-dir pkg/nodejs
node examples/nodejs 444444
```

### Browser example

```shell
wasm-pack build www --target bundler --out-dir pkg/web
cd examples/webpack
npm install
npm start
```
