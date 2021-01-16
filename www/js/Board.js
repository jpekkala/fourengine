export default class Board {
    constructor(container, options = {}) {
        this.container = container;
        this.cols = options.cols || 7;
        this.rows = options.rows || 6;
        this.cellSize = options.cellSize || 75;
        this.boardColor = options.boardColor || 'black';
        this.position = options.position || [];
        this.animationSpeedMs = options.animationSpeedMs || 200;
        this.drawBoard();
    }

    getSvgDisc({ col, row, animate }) {
        const value = this.position[col * this.cols + row];
        if (!value) return;
        const axis = this.cellSize / 2;
        const radius = axis * 0.9;
        const cy = this.rows * this.cellSize - axis - row * this.cellSize;

        let color;
        switch (value) {
            case 1: color = 'player_one'; break;
            case 2: color = 'player_two'; break;
            case 3: color = 'winning_disc'; break;
            default: return;
        }

        const circle = this.getNode('circle', {
            cx: axis,
            cy,
            fill: `url(#${color})`,
            r: radius,
        })

        if (animate) {
            const animation = this.getNode('animate', {
                attributeName: 'cy',
                from: -axis,
                to: cy,
                // TODO: adjust time
                dur: `${(this.rows - row) * this.animationSpeedMs}ms`,
                begin: '0s',
                fill: 'freeze'
            })
            circle.appendChild(animation)
        }

        return circle
    }

    drawBoard() {
        this.container.innerHTML = '';
        const width = this.cols * this.cellSize;
        const height = this.rows * this.cellSize;
        const axis = this.cellSize / 2;
        const skewY = this.cellSize / 10;
        const skewX = -skewY;

        const html = `
            <div style="width:${width}px;height:${height}px;margin-top:20px;border:${Math.round(axis/2)}px solid ${this.boardColor};border-radius:${axis}px;">
                <svg width="${width}px" viewBox="0 0 ${width} ${height}" id="svg_view">
                    <defs>
                        <pattern id="grid_pattern" patternUnits="userSpaceOnUse" width="${this.cellSize}" height="${this.cellSize}">
                            <circle cx="${axis}" cy="${axis}" r="${axis*0.8}" fill="black" />
                        </pattern>
                        <mask id="column_mask">
                            <rect width="${this.cellSize}" height="${height}" fill="white" />
                            <rect width="${this.cellSize}" height="${height}" fill="url(#grid_pattern)" />
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
                </svg>
            </div>`;

        const board = this.stringToHTML(html);
        this.container.appendChild(board);
        const svgView = board.querySelector('#svg_view');

        for (let i = 0; i < this.cols; i++) {
            const column = this.getNode('svg', {
                x: this.cellSize * i,
                y: 0,
                id: `column_${i}`
            })
            svgView.appendChild(column);

            for (let j = 0; j < this.rows; j++) {
                const disc = this.getSvgDisc({ col: i, row: j });
                if (disc) column.appendChild(disc);
            }

            const mask = this.getNode('rect', {
                width: this.cellSize,
                height,
                fill: this.boardColor,
                mask: 'url(#column_mask)'
            })
            column.appendChild(mask);
        }
    }

    animateMove({ col, row, newPosition, isWinningMove = false, moveType = 'drop', value }) {
        this.position[col * this.cols + row] = value;
        const animatedDisc = this.getSvgDisc({ col, row, animate: true });
        const column = this.container.querySelector(`#column_${col}`);
        column.insertBefore(animatedDisc, column.firstChild);
    }

    setPosition(pos) {
        console.log('Setting position:', pos)
        this.position = pos
        this.drawBoard()
    }

    stringToHTML(str) {
        const parser = new DOMParser();
        const doc = parser.parseFromString(str, 'text/html');
        return doc.body.firstChild;
    }

    getNode(node, props) {
        const n = document.createElementNS("http://www.w3.org/2000/svg", node);
        for (const p in props) {
            n.setAttributeNS(null, p, props[p]);
        }
        return n
    }
}