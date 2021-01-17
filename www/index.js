import Board from './js/Board';
import Game from './js/Game';

window.Game = Game;

const div = document.querySelector('#c4_board');
const board = new Board(div);

window.solve = async function(variation) {
    document.getElementById('solution').innerHTML = 'Solving...';
    setFormDisabled(true);
    try {
        console.log('Solving variation', variation);
        const game = new Game(variation);
        board.setPosition(game.getCellMatrix());
        const solution = await game.solve();
        console.log('Solution is', solution);
        let description = '';
        description += `Score: ${solution.score}<br/>`;
        description += `Work count: ${solution.workCount}<br/>`;
        description += `Elapsed time: ${solution.duration} s<br/>`;
        description += `Nodes per second: ${solution.nps}<br/>`;
        document.getElementById('solution').innerHTML = description;
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
