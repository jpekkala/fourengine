// Web workers don't have access to console. Instead we'll send messages to the main thread and print them there
self.console = {
    log() {
        postMessage({
            type: 'log',
            args: [...arguments],
        });
    },

    error() {
        postMessage({
            type: 'error',
            args: [...arguments],
        });
    }
}

const enginePromise = import("fourengine").then(wasm => {
    return new wasm.Engine();
}).catch(error => {
    console.error('Failed to load wasm', error);
    throw error;
});

self.onmessage = async function (e) {
    try {
        const engine = await enginePromise;
        const { variation } = e.data;
        const start = performance.now();
        const struct = engine.solve(variation);
        const duration = (performance.now() - start) / 1000;
        const message = {
            type: 'solution',
            duration,
            score: struct.getScore(),
            workCount: struct.getWorkCount(),
            nps: Math.round(struct.getWorkCount() / duration),
        }
        postMessage(message);
    } catch (error) {
        postMessage({
            type: 'reject',
            error,
        })
    }
};
