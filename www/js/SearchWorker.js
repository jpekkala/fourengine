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
    const duration = (performance.now() - start) / 1000;
    const result = {
        duration,
        score: struct.get_score(),
        workCount: struct.get_work_count(),
        nps: Math.round(struct.get_work_count() / duration),
    }
    postMessage(result);
};
