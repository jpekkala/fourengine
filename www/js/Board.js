import Game from './Game';

export default class Board {

    constructor(options = {}) {
        this.container = options.container || document.querySelector('#c4_board');
        this.cols = options.cols || 7;
        this.rows = options.rows || 6;
        this.cellSize = options.cellSize || 75;
        this.boardColor = options.boardColor || 'black';
        this.animationSpeedMs = options.animationSpeedMs || 200;
        this.autoSolve = options.hasOwnProperty('autoSolve') ? options.autoSolve : false;

        this.game = new Proxy(new Game(options.variation), {
            set: (target, key, value) => {
                target[key] = value;
                if (key == 'variation') {
                    this.setFormDisabled(false);
                    this.variationInput.value = value;
                }
                return true;
            }
        });
        this.time = Date.now();
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
            const svgTime = this.boardView.getCurrentTime() || (Date.now() - this.time) / 1000;
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

            animation.addEventListener('endEvent', () => {
                this.animating--;
                this.transformDiscs();
            });

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

    getGameBtn(type) {
        let btnStyle = 'width: 50px; height: 30px; display: flex; align-items: center; justify-content: center;';
        if (type == 'undo' || type == 'fast-forward') {
            btnStyle += 'transform: rotateY(180deg);';
        }
        
        if (type == 'undo' || type == 'redo') {
            return this.stringToHTML(`<button style="${btnStyle}" id="${type}_move">
                <svg height="25" viewBox="-265 388.9 64 64" xmlns="http://www.w3.org/2000/svg">
                    <path d="M-242.6,407l26.1,15.1c0.6,0.4,0.6,1.2,0,1.6l-26.1,15.1c-0.6,0.4-1.4-0.1-1.4-0.8v-30.2C-244,407.1-243.2,406.7-242.6,407z" />
                </svg>
            </button>`);

        } else if(type == 'rewind' || type == 'fast-forward') {
            return this.stringToHTML(`<button style="${btnStyle}" id="${type}_move">
                <svg height="15" viewBox="0 0 554.239 554.239" xmlns="http://www.w3.org/2000/svg">
                    <path d="M512.084,86.498L314.819,215.25V110.983c0-26.805-18.874-37.766-42.155-24.479L17.46,253.065
                        c-23.28,13.287-23.28,34.824,0,48.11l255.204,166.562c23.281,13.286,42.155,2.325,42.155-24.48V338.985l197.266,128.753
                        c23.28,13.286,42.154,2.325,42.154-24.48V110.983C554.239,84.178,535.365,73.217,512.084,86.498z" />
                </svg>
            </button>`);
        }
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

        const board = this.stringToHTML(`<div style="width:${width}px;">
            <div style="margin:0px ${borderWidth}px;">
                <input type="checkbox" id="auto_solve" name="auto_solve" ${this.autoSolve ? 'checked' :''}>
                <label for="auto_solve">Auto solve</label>
            </div>
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
            <div style="width:${width}px;display:flex;flex-direction:row;justify-content:space-between;margin:0 ${borderWidth}px;">
                <div id="game_buttons_left" style="display:flex;flex-direction:row;align-items:center;"></div>
                <div id="game_buttons_right" style="display:flex;flex-direction:row;"></div>
            </div>
            <div id="solution" style="margin:10px ${borderWidth}px;"></div>
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

        this.attachButtons();
    }

    pop(column) {
        // Replace column
    }

    mouseHandler(column, e) {
        if (this.solving || this.game.hasWon()) return;

        if (e.which === 1) {
            console.log(`Clicked column ${column + 1}`);
            this.drop(column);
        } else if (e.which === 3) {
            console.log(`Right-clicked column ${column + 1}`);
        }
    }

    async solve() {
        if (this.solving) return;
        this.solving = true;
        document.getElementById('solution').innerHTML = 'Solving...';
        this.setFormDisabled(true);
        try {
            const solution = await this.game.solve();
            this.transformDiscs();
            this.printSolution(solution);
            console.log('Solution is', solution);
        } finally {
            this.solving = false;
            this.setFormDisabled(false);
        }
    }

    attachButtons() {
        const autoSolveChkbox = document.getElementById('auto_solve');
        autoSolveChkbox.addEventListener('click', e => {
            this.autoSolve = e.target.checked;
        });

        this.variationInput = this.stringToHTML(`<input autocomplete="off" style="padding:5px 2px;font-size:14px;width:220px;box-sizing:border-box;${isFirefox ? 'height:28px;' : ''}" id="variation" />`);
        if (this.game.variation) {
            this.variationInput.value = this.game.variation;
        }
        const solveButton = this.stringToHTML(`<button style="height:30px;" id="solve_button">Solve</button>`);
        solveButton.addEventListener('click', e => {
            const variation = document.getElementById('variation').value;
            this.setVariation(variation);
            this.solve();
        });

        const gameButtonsLeft = document.getElementById('game_buttons_left');
        gameButtonsLeft.appendChild(this.variationInput);
        gameButtonsLeft.appendChild(solveButton);

        const gameButtonsRight = document.getElementById('game_buttons_right');
        const rewindBtn = this.getGameBtn('rewind');
        const undoBtn = this.getGameBtn('undo');
        const redoBtn = this.getGameBtn('redo');
        const ffBtn = this.getGameBtn('fast-forward');

        rewindBtn.addEventListener('click', e => {
            this.setVariation('');
        });
        undoBtn.addEventListener('click', e => {
            this.game.undo();
            this.drawBoard();
            if (this.autoSolve) {
                this.solve();
            }
        });
        redoBtn.addEventListener('click', e => {
            this.game.redo();
            this.drawBoard();
            if (this.autoSolve) {
                this.solve();
            }
        });
        ffBtn.addEventListener('click', e => {
            this.game.fastForward();
            this.drawBoard();
            if (this.autoSolve) {
                this.solve();
            }
        });
        gameButtonsRight.appendChild(rewindBtn);
        gameButtonsRight.appendChild(undoBtn);
        gameButtonsRight.appendChild(redoBtn);
        gameButtonsRight.appendChild(ffBtn);
        this.setFormDisabled(false);
    }

    setFormDisabled(disabled) {
        document.getElementById('solve_button').disabled = disabled;
        document.getElementById('variation').disabled = disabled;
        document.getElementById('rewind_move').disabled = disabled || !this.game.canUndo;
        document.getElementById('undo_move').disabled = disabled || !this.game.canUndo;
        document.getElementById('redo_move').disabled = disabled || !this.game.canRedo;
        document.getElementById('fast-forward_move').disabled = disabled || !this.game.canRedo;
    }

    printSolution(solution) {
        const description = `<div>
            <span style="font-weight:bold;">Score:</span> ${solution.score}<br/>
            <span style="font-weight:bold;">Work count:</span> ${solution.workCount.toLocaleString()}<br/>
            <span style="font-weight:bold;">Elapsed time:</span> ${solution.duration.toFixed(3)} s<br/>
            <span style="font-weight:bold;">Nodes per second:</span> ${solution.nps.toLocaleString()}<br/>
        </div>`;
        document.getElementById('solution').innerHTML = description;
    }

    drop(column) {
        if (!this.game.canDrop(column)) return;
        const pos = this.game.drop(column);
        if (this.autoSolve) {
            this.solve();
        }
        const animatedDisc = this.getSvgDisc({ ...pos, animate: true });
        const boardColumn = document.getElementById(`column_${column}`);
        boardColumn.insertBefore(animatedDisc, boardColumn.firstChild);
    }

    setVariation(variation) {
        console.log('Setting variation:', variation);
        this.game.setVariation(variation);
        this.drawBoard();
        if (this.autoSolve) {
            this.solve();
        }
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

const isFirefox = typeof InstallTrigger !== 'undefined';