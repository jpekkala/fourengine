import GameModel from './js/GameModel';
import SvgBoard from './js/SvgBoard';

const game = new GameModel();
const svgBoard = new SvgBoard({
    game,
    container: document.querySelector('#c4_board'),
});
svgBoard.drawBoard();