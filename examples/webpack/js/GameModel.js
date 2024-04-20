// TODO: This is fetched twice because the web worker also needs it. Figure out how to share it with web worker
import * as wasm from '../../../wasm/bundler/fourengine_wasm.js';

const BOARD_WIDTH = 7;
const BOARD_HEIGHT = 6;

/**
 * The game state which consists of the current position and methods to manipulate that. This class wraps and hides the
 * underlying Wasm types for ease of use.
 */
export default class GameModel {

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
        if (this.variation === variation) {
            return;
        }
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
        const row = this.position.getHeight(column);
        this.setVariation(this.variation + ch);
        return {
            column,
            row,
        };
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

    setSolver(solver) {
        this.solver = solver;
    }

    async solve() {
        if (this.solver) {
            return await this.solver.solve(this.variation);
        } else {
            return {
                score: 'Unknown',
            };
        }
    }

    stopSolve() {
        this.solver?.stop();
    }
}
