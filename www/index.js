import SearchWorker from 'worker-loader!./js/SearchWorker';
// TODO: This is fetched twice because the web worker also needs it. Figure out how to share it with web worker
import * as wasm from 'fourengine';

const worker = new SearchWorker();
worker.addEventListener('message', function(e) {
    console.log('Worker message', e.data);
});

window.solve = function(variation) {
    worker.postMessage({ variation, });
}

window.showPosition = function(variation) {
  wasm.show_position(variation);
};
