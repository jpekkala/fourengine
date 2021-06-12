import GameModel from './js/GameModel';
import Solver from './js/Solver';
import SvgBoard from './js/SvgBoard';

const game = new GameModel();
game.setSolver(new Solver());

const svgBoard = new SvgBoard({
    game,
    container: document.querySelector('#c4_board'),
});
svgBoard.drawBoard();