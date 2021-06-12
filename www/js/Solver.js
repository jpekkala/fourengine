import EngineWorker from 'worker-loader!./EngineWorker';

/**
 * Starts and manages web workers for solving
 */
export default class Solver {
    constructor() {
        this.resetWorker();
    }

    resetWorker() {
        if (this.worker) {
            console.log('Terminating worker');
            this.worker.terminate();
            this.onReject?.('Interrupted');
        }
        this.worker = new EngineWorker();
        this.worker.addEventListener('message', e => this.onWorkerMessage(e));
        this.solving = false;
    }

    onWorkerMessage(event) {
        const type = event.data.type;
        if (type === 'log') {
            console.log(...event.data.args);
        } else if (type === 'error') {
            console.error(...event.data.args);
        } else if (type === 'reject') {
            this.solving = false;
            this.onReject?.(event.data.error);
        } else if (type === 'solution') {
            this.solving = false;
            this.onResolve?.(event.data);
        } else {
            console.error('Unhandled worker message', event);
        }
    }

    async solve(variation) {
        if (this.solving) {
            this.resetWorker();
        }
        this.solving = true;

        return new Promise((resolve, reject) => {
            this.onResolve = resolve;
            this.onReject = reject;

            console.log('Sending variation', variation, 'to web worker');
            this.worker.postMessage({ variation, });
        });
    }

    stop() {
        if (this.solving) {
            this.resetWorker();
        }
    }
}