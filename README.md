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