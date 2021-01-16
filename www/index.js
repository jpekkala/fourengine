import SearchWorker from 'worker-loader!./js/SearchWorker';
// TODO: This is fetched twice because the web worker also needs it. Figure out how to share it with web worker
import * as wasm from 'fourengine';
import Board from './js/Board';

const worker = new SearchWorker();
worker.addEventListener('message', function(e) {
    const solution = e.data;
    console.log('Solution is', solution);
    setFormDisabled(false);

    let description = '';
    description += `Score: ${solution.score}<br/>`;
    description += `Work count: ${solution.workCount}<br/>`;
    description += `Elapsed time: ${solution.duration} s<br/>`;
    description += `Nodes per second: ${solution.nps}<br/>`;

    document.getElementById('solution').innerHTML = description;
});

window.onload = function() {
    const div = document.querySelector('#c4_board');
    const board = new Board(div);
};

window.solve = function(variation) {
    document.getElementById('solution').innerHTML = 'Solving...';
    setFormDisabled(true);
    worker.postMessage({ variation, });
}

window.showPosition = function(variation) {
  wasm.show_position(variation);
};

window.onSolveButtonClick = function() {
    const variation = document.getElementById('variation').value;
    window.solve(variation);
};

function setFormDisabled(disabled) {
    document.getElementById('solve-button').disabled = disabled;
    document.getElementById('variation').disabled = disabled;
}
