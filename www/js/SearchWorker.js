let wasm;
let engine;

import("fourengine").then(wasmModule => {
    wasm = wasmModule;
    engine = wasm.init_engine();
})

self.onmessage = function (e) {
    if (!wasm) {
        postMessage({ error: "wasm not ready" });
        return;
    }
    const { variation } = e.data;
    const start = performance.now();
    const struct = wasm.solve(engine, variation);
    const elapsedTime = performance.now() - start;
    const result = {
        duration: elapsedTime,
        score: struct.get_score(),
        workCount: struct.get_work_count(),
        nps: struct.get_work_count() / elapsedTime * 1000,
    }
    postMessage(result);
};
