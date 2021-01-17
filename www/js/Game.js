import SearchWorker from 'worker-loader!./SearchWorker';
// TODO: This is fetched twice because the web worker also needs it. Figure out how to share it with web worker
import * as wasm from 'fourengine';

export default class Game {

    constructor(variation = '') {
        this.setVariation(variation);
    }

    setVariation(variation) {
        this.variation = variation;
        this.position = new wasm.Position(variation);
    }

    getCell(x, y) {
        return this.position.getCell(x, y);
    }

    hasWon() {
        return this.position.hasWon();
    }

    canDrop(column) {
        return this.position.canDrop(column);
    }

    drop(column) {
        const ch = String(column + 1);
        this.setVariation(this.variation + ch);
    }

    undo() {
        const previous = this.variation.substring(0, this.variation.length - 1);
        this.setVariation(previous);
    }

    async solve() {
        return new Promise((resolve, reject) => {
            const worker = new SearchWorker();
            worker.addEventListener('message', function (e) {
                if (e.data.error) {
                    reject(e.data.error);
                } else {
                    resolve(e.data);
                }
            });
            worker.postMessage({ variation: this.variation });
        })
    }
}
