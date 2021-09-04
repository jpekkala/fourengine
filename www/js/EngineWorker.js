/**
 * Runs the Connect-4 engine in a separate thread. This file wraps and prepares the engine which itself is in Wasm.
 */

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
};

const wasmPromise = import('../pkg/fourengine_wasm.js').then(wasm => {
    console.log('Wasm loaded');
    return wasm;
}).catch(error => {
    console.error('Failed to load wasm', error);
    throw error;
});

// TODO: Load books outside of worker so that it can be shared between workers
function fetchBook(ply) {
    return fetch(`7x6-ply${ply}.txt`).then(response => {
        if (!response.ok) {
            throw Error(`${response.status} ${response.statusText}`);
        }
        console.log('Book loaded');
        return response.text();
    }).catch(err => {
        console.error('Error loading book:', err.message);
        return '';
    });
}

const enginePromise = Promise.all([wasmPromise, fetchBook(4), fetchBook(8)]).then(values => {
    const [wasm, ply4, ply8] = values;
    const engine = new wasm.Engine();
    const book = new wasm.Book();
    book.includeLines(ply4);
    book.includeLines(ply8);
    engine.setBook(book);
    return engine;
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
        };
        postMessage(message);
    } catch (error) {
        postMessage({
            type: 'reject',
            error: error.message,
        });
    }
};
