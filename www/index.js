import * as wasm from "fourengine";

window.solve = function(variation) {
    console.log(`Solving variation ${variation}...`);
    // Note that the duration is not fully fair because it includes engine initialization
    const start = Date.now();
    const workCount = wasm.solve(variation);
    const elapsedTime = Date.now() - start;
    console.log('Elapsed time:', elapsedTime);
    console.log('Work count is:', workCount);
    console.log('Nodes per second:', workCount / (elapsedTime / 1000));
}
