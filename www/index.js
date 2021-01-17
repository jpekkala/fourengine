import Board from './js/Board';

const board = new Board();

window.solve = async function(variation) {
    setFormDisabled(true);
    try {
        console.log('Solving variation', variation);
        board.setVariation(variation);
        await board.solve();
    } finally {
        setFormDisabled(false);
    }
}

window.onSolveButtonClick = function() {
    const variation = document.getElementById('variation').value;
    window.solve(variation);
};

function setFormDisabled(disabled) {
    document.getElementById('solve-button').disabled = disabled;
    document.getElementById('variation').disabled = disabled;
}
