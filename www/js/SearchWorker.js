const enginePromise = import("fourengine").then(wasm => {
    return new wasm.Engine();
}).catch(error => {
    postMessage({ error, })
    throw error;
});

self.onmessage = async function (e) {
    const engine = await enginePromise;
    const { variation } = e.data;
    const start = performance.now();
    const struct = engine.solve(variation);
    const duration = (performance.now() - start) / 1000;
    const result = {
        duration,
        score: struct.getScore(),
        workCount: struct.getWorkCount(),
        nps: Math.round(struct.getWorkCount() / duration),
    }
    postMessage(result);
};
