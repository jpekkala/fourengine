import SearchWorker from 'worker-loader!./SearchWorker';
// TODO: This is fetched twice because the web worker also needs it. Figure out how to share it with web worker
import * as wasm from 'fourengine';

const BOARD_WIDTH = 7;
const BOARD_HEIGHT = 6;

export default class Game {

    constructor(variation = '') {
        this.savedState = '';
        this.setVariation(variation);
    }

    get canUndo() {
        return this.variation.length > 0;
    }

    get canRedo() {
        return this.savedState > this.variation;
    }

    setVariation(variation) {
        if (!this.savedState.startsWith(variation)) {
            this.savedState = variation;
        }
        this.variation = variation;
        this.position = new wasm.Position(variation);
    }

    getCell(x, y) {
        return this.position.getCell(x, y);
    }

    getCellMatrix() {
        const matrix = new Array(BOARD_WIDTH * BOARD_HEIGHT);
        for (let x = 0; x < BOARD_WIDTH; x++) {
            for (let y = 0; y < BOARD_HEIGHT; y++) {
                matrix[x * BOARD_HEIGHT + y] = this.getCell(x, y);
            }
        }
        return matrix;
    }

    hasWon() {
        return this.position.hasWon();
    }

    isWinningCell(x, y) {
        return this.position.isWinningCell(x, y);
    }

    canDrop(column) {
        return this.position.canDrop(column);
    }

    drop(column) {
        const ch = String(column + 1);
        if (!this.canDrop(column)) {
            throw Error(`Cannot drop in column "${ch}" because it is full`);
        }
        this.variation += ch;
        if (!this.savedState.startsWith(this.variation)) {
            this.savedState = this.variation;
        }
        const row = this.position.getHeight(column);
        this.position = this.position.drop(column);
        return {
            column,
            row,
        }
    }

    undo() {
        const previous = this.variation.substring(0, this.variation.length - 1);
        this.setVariation(previous);
    }

    redo() {
        const next = this.savedState.substring(0, this.variation.length + 1);
        this.setVariation(next);
    }

    fastForward() {
        this.setVariation(this.savedState);
    }

    async solve() {
        return new Promise((resolve, reject) => {
            if (this.worker) {
                this.worker.terminate();
            }
            this.worker = new SearchWorker();

            this.worker.addEventListener('message', function (e) {
                const type = e.data.type;
                if (type === 'log') {
                    console.log(...e.data.args);
                } else if (type === 'error') {
                    console.error(...e.data.args);
                } else if (type === 'reject') {
                    reject(e.data.error);
                } else if (type === 'solution') {
                    this.solving = false;
                    resolve(e.data);
                } else {
                    console.error('Unhandled worker message', e);
                }
            });
            this.worker.postMessage({ variation: this.variation });
        })
    }
}
