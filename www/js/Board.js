import Game from './Game';

export default class Board {

    constructor(options = {}) {
        this.container = options.container || document.querySelector('#c4_board');
        this.cols = options.cols || 7;
        this.rows = options.rows || 6;
        this.cellSize = options.cellSize || 75;
        this.boardColor = options.boardColor || 'black';
        this.animationSpeedMs = options.animationSpeedMs || 200;
        this.game = new Game(options.variation);
        this.drawBoard();
    }

    get position() {
        return this.game.getCellMatrix();
    }

    getSvgDisc({ column, row, animate, animationSettings = {} }) {
        const value = this.position[column * this.rows + row];
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
            id: `disc_${column}${row}`
        });

        if (animate) {
            const svgTime = this.boardView.getCurrentTime();
            const animFrom = animationSettings.from || -axis;
            const animTo = animationSettings.to || cy;
            const duration = (cy + axis) / this.cellSize * this.animationSpeedMs;
            const animation = this.getNode('animate', {
                attributeName: 'cy',
                from: animFrom,
                to: animTo,
                dur: `${duration}ms`,
                begin: `${svgTime}s`,
                fill: 'freeze'
            })

            animation.onend = () => {
                this.animating--;
                this.transformDiscs();
            }

            circle.appendChild(animation);
            this.animating++;
        }

        return circle;
    }

    getSolutionIndicator(type) {
        const width = this.cellSize * 0.7;
        const svgHeight = this.cellSize * 0.20;
        const svgWidth = this.cellSize * 0.5;
        const colors = {
            loading: 'gray',
            red: '#F40000',
            blue: '#526BE1',
            draw: '#FBFB00',
        }

        const elem = this.stringToHTML(`<div style="width:${width}px;background:${colors[type] || 'black'};border-radius:4px;box-shadow: -1px 0 3px 0 rgba(0, 0, 0, .5);display:flex;justify-content:center;align-items:center;"></div>`);

        if (type === 'loading') {
            const outerAnim = `
                <animate 
                    attributeName="r"
                    from="15" to="15"
                    begin="0s" dur="0.8s"
                    values="15;9;15" calcMode="linear"
                    repeatCount="indefinite"
                />
                <animate
                    attributeName="fill-opacity"
                    from="1" to="1"
                    begin="0s" dur="0.8s"
                    values="1;.5;1" calcMode="linear"
                    repeatCount="indefinite"
                />
            `;

            const loader = this.stringToHTML(`
                <svg width="${svgWidth}" height="${svgHeight}" viewBox="0 0 120 20" xmlns="http://www.w3.org/2000/svg" fill="white">
                    <circle cx="15" cy="10" r="15">
                        ${outerAnim}
                    </circle>
                    <circle cx="60" cy="10" r="9" fill-opacity="0.3">
                        <animate
                            attributeName="r"
                            from="9" to="9"
                            begin="0s" dur="0.8s"
                            values="9;15;9" calcMode="linear"
                            repeatCount="indefinite"
                        />
                        <animate
                            attributeName="fill-opacity"
                            from="0.5" to="0.5"
                            begin="0s" dur="0.8s"
                            values=".5;1;.5" calcMode="linear"
                            repeatCount="indefinite"
                        />
                    </circle>
                    <circle cx="105" cy="10" r="15">
                        ${outerAnim}
                    </circle>
                </svg>
            `);

            elem.appendChild(loader);
        }

        return elem;
    }

    transformDiscs() {
        if (!this.game.hasWon() || this.animating) return;
        for (let x = 0; x < this.cols; x++) {
            for (let y = 0; y < this.rows; y++) {
                if (!this.game.isWinningCell(x, y)) continue;
                const cell = this.container.querySelector(`#disc_${x}${y}`);
                cell.setAttributeNS(null, 'fill', 'url(#winning_disc)');
            }
        }
    }

    drawBoard() {
        this.animating = 0;
        this.container.innerHTML = '';
        const width = this.cols * this.cellSize;
        const height = this.rows * this.cellSize;
        const axis = this.cellSize / 2;
        const skewY = this.cellSize / 10;
        const skewX = -skewY;
        const borderWidth = Math.round(axis / 2);

        const board = this.stringToHTML(`<div>
            <div style="width:${width}px;height:${height}px;margin-top:20px;border:${borderWidth}px solid ${this.boardColor};border-radius:${axis}px;" oncontextmenu="return false;">
                <svg width="${width}px" viewBox="0 0 ${width} ${height}" id="board_view">
                    <defs>
                        <pattern id="grid_pattern" patternUnits="userSpaceOnUse" width="${this.cellSize}" height="${this.cellSize}">
                            <circle cx="${axis}" cy="${axis}" r="${axis * 0.8}" fill="black" />
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
            </div>
            <div id="column_solutions" style="width:${width}px;margin:10px ${borderWidth}px;display:flex;flex-direction:row;"></div>
        </div>`);
        this.container.appendChild(board);
        this.boardView = board.querySelector('#board_view');
        this.solutionsView = board.querySelector('#column_solutions');

        for (let i = 0; i < this.cols; i++) {
            // Board columns
            const column = this.getNode('svg', {
                x: this.cellSize * i,
                y: 0,
                id: `column_${i}`
            });

            column.addEventListener('mousedown', e => this.mouseHandler(i, e));
            this.boardView.appendChild(column);

            for (let j = 0; j < this.rows; j++) {
                const disc = this.getSvgDisc({ column: i, row: j });
                if (disc) column.appendChild(disc);
            }

            const mask = this.getNode('rect', {
                width: this.cellSize,
                height,
                fill: this.boardColor,
                mask: 'url(#column_mask)'
            });
            column.appendChild(mask);

            // Column solutions
            const solution = this.stringToHTML(`
                <div id="solution_${i}" style="display:flex;justify-content:center;width:${this.cellSize}px;"></div>
            `);

            this.solutionsView.appendChild(solution);
        }
    }

    pop(column) {
        // Replace column
    }

    mouseHandler(column, e) {
        if (this.solving || this.game.hasWon()) return;

        if (e.which === 1) {
            console.log(`Clicked column ${column}`);
            this.drop(column);
        } else if (e.which === 3) {
            console.log(`Right-clicked column ${column}`);
        }
    }

    async solve() {
        this.solving = true;
        document.getElementById('solution').innerHTML = 'Solving...';
        try {
            const solution = await this.game.solve();
            this.transformDiscs();
            console.log('Solution is', solution);
            let description = '';
            description += `Score: ${solution.score}<br/>`;
            description += `Work count: ${solution.workCount}<br/>`;
            description += `Elapsed time: ${solution.duration} s<br/>`;
            description += `Nodes per second: ${solution.nps}<br/>`;
            document.getElementById('solution').innerHTML = description;
        } finally {
            this.solving = false;
        }
    }

    drop(column) {
        if (!this.game.canDrop(column)) return;
        const pos = this.game.drop(column);
        this.solve();
        const animatedDisc = this.getSvgDisc({ ...pos, animate: true });
        const boardColumn = this.container.querySelector(`#column_${column}`);
        boardColumn.insertBefore(animatedDisc, boardColumn.firstChild);
    }

    setVariation(variation) {
        console.log('Setting variation:', variation);
        this.game.setVariation(variation);
        this.drawBoard();
    }

    stringToHTML(str) {
        const parser = new DOMParser();
        const doc = parser.parseFromString(str, 'text/html');
        return doc.body.firstChild;
    }

    getNode(node, props) {
        const n = document.createElementNS('http://www.w3.org/2000/svg', node);
        for (const p in props) {
            n.setAttributeNS(null, p, props[p]);
        }
        return n;
    }
}