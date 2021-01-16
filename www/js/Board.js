export default class Board {
    constructor(container, options = {}) {
        this.container = container;
        this.cols = options.cols || 7;
        this.rows = options.rows || 6;
        this.cellSize = options.cellSize || 75;
        this.boardColor = options.boardColor || 'black';
        this.position = options.position || [];
        this.drawBoard();
    }

    getSvgDisc(col, row) {
        // TODO: flat array?
        const value = this.position[col] && this.position[col][row];
        if (!value) return '';
        const axis = this.cellSize / 2;
        const radius = axis * 0.9;
        const cy = this.rows * this.cellSize - axis - row * this.cellSize;

        let disc;
        switch (value) {
            case 1: disc = 'player_one'; break;
            case 2: disc = 'player_two'; break;
            case 3: disc = 'winning_disc'; break;
            default: return '';
        }

        return `<circle cx="${axis}" cy="${cy}" r="${radius}" fill="url(#${disc})" />`;
    }

    drawBoard() {
        const width = this.cols * this.cellSize;
        const height = this.rows * this.cellSize;
        const axis = this.cellSize / 2;
        const skewX = -this.cellSize / 10;
        const skewY = this.cellSize / 10;

        const template = document.createElement('template');
        template.innerHTML = `
            <div style="width:${width}px;height:${height}px;margin-top:20px;border:${Math.round(axis/2)}px solid ${this.boardColor};border-radius:${axis}px;">
                <svg width="${width}px" viewBox="0 0 ${width} ${height}">
                    <defs>
                        <pattern id="grid_pattern" patternUnits="userSpaceOnUse" width="${this.cellSize}" height="${this.cellSize}">
                            <circle cx="${axis}" cy="${axis}" r="${axis*0.8}" fill="black" />
                        </pattern>
                        <mask id="column_mask">
                            <rect width="${this.cellSize}" height="${height}" fill="white"></rect>
                            <rect width="${this.cellSize}" height="${height}" fill="url(#grid_pattern)"></rect>
                        </mask>
                        <radialGradient id="player_one" gradientTransform="skewX(${skewX}) skewY(${skewY})">
                            <stop offset="0%" stop-color="#536ce2" />
                            <stop offset="100%" stop-color="#122997" />
                        </radialGradient>
                        <radialGradient id="player_two" gradientTransform="skewX(${skewX}) skewY(${skewY})" >
                            <stop offset="0%" stop-color="red" />
                            <stop offset="100%" stop-color="#660000" />
                        </radialGradient>
                        <radialGradient id="winning_disc" gradientTransform="skewX(${skewX}) skewY(${skewY})" >
                            <stop offset="0%" stop-color="yellow" />
                            <stop offset="100%" stop-color="#c6c600" />
                        </radialGradient>
                    </defs>
                    ${Array.from(Array(this.cols)).map((_, i) => `
                        <svg x="${this.cellSize*i}" y="0">
                            ${Array.from(Array(this.rows)).map((_, j) => this.getSvgDisc(i, j))}
                            <rect width="${this.cellSize}" height="${height}" fill="${this.boardColor}" mask="url(#column_mask)" />
                        </svg>`)}
                </svg>
            </div>`;
        this.container.innerHTML = '';
        this.container.appendChild(template.content.cloneNode(true));
    }

    animateMove({ x, y, newPosition, isWinningMove = false, moveType = 'drop' }) {

    }

    setPosition(pos) {
        console.log('Setting position:', pos)
        this.position = pos
        this.drawBoard()
    }
}