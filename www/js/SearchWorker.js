let wasm;
let engine;

const wasmReady = import("fourengine").then(wasmModule => {
    wasm = wasmModule;
    engine = wasm.init_engine();
}).catch(error => {
    postMessage({ error, })
})

self.onmessage = async function (e) {
    await wasmReady;
    const { variation } = e.data;
    const start = performance.now();
    const struct = wasm.solve(engine, variation);
    const duration = (performance.now() - start) / 1000;
    const result = {
        duration,
        score: struct.get_score(),
        workCount: struct.get_work_count(),
        nps: Math.round(struct.get_work_count() / duration),
    }
    postMessage(result);
};
