import SvgBoard from './js/SvgBoard';

const board = new SvgBoard({
    container: document.querySelector('#c4_board'),
});
board.drawBoard();