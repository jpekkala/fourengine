const fs = require('node:fs')
const fsPromises = require('node:fs/promises')
const wasm = require('./nodejs')

const BOARD_WIDTH = 7
const BOARD_HEIGHT = 6

const BOOK_FILES = [
    './books/7x6-ply4.txt',
    './books/7x6-ply8.txt'
]

async function loadBook() {
    const bookDatas = await Promise.all(BOOK_FILES.map(filename => fsPromises.readFile(filename, 'utf8')))

    const book = new wasm.Book()
    for (const data of bookDatas) {
        book.includeLines(data)
    }
    return book
}

function loadBookSync() {
    const book = new wasm.Book()
    for (const filename of BOOK_FILES) {
        const data = fs.readFileSync(filename, 'utf8')
        book.includeLines(data)
    }
    return book
}

class Fourengine {

    constructor() {
        this.engine = new wasm.Engine()
        this.bookEnabled = false
    }

    async enableBook() {
        if (this.bookEnabled) {
            return
        }

        const book = await loadBook()
        this.engine.setBook(book)
        this.bookEnabled = true
    }

   enableBookSync() {
        if (this.bookEnabled) {
            return
        }

        const book = loadBookSync()
        this.engine.setBook(book)
        this.bookEnabled = true
    }

    solve(position) {
        if (typeof position === 'string') {
            position = new Position(position)
        }

        const start = performance.now();
        const solution = this.engine.solve(position.variation)
        const duration = (performance.now() - start) / 1000;
        return {
            score: solution.getScore(),
            duration: duration.toFixed(2) + ' seconds',
            workCount: solution.getWorkCount(),
            nps: Math.round(solution.getWorkCount() / duration)
        }
    }
}

class Position {
    constructor(variation) {
        this.variation = variation;
        try {
            this.wasmPosition = new wasm.Position(this.variation)
        } catch (e) {
            throw Error('Invalid position')
        }
    }

    getCell(x, y) {
        const cell = this.wasmPosition.getCell(x, y)
        switch (cell) {
            case 0:
                return '.'
            case 1:
                return 'X'
            case 2:
                return 'O'
            default:
                return '?'
        }
    }

    hasWon() {
        return this.wasmPosition.hasWon()
    }

    isWinningCell(x, y) {
        return this.wasmPosition.isWinningCell(x, y)
    }

    canDrop() {
        return this.wasmPosition.canDrop()
    }

    getColumnHeight(x) {
        return this.wasmPosition.getHeight(x)
    }

    toString() {
        let string = ''
        for (let y = BOARD_HEIGHT - 1; y >= 0; y--) {
            for (let x = 0; x < BOARD_WIDTH; x++) {
                string += this.getCell(x, y)
            }
            string += '\n'
        }
        return string
    }
}

async function yesNoQuestion(rl, question) {
    let answer = await rl.question(question)
    answer = answer.toLowerCase()
    return answer === 'y' || answer === 'yes'
}

async function main() {
    const readline = require('node:readline/promises')
    const { stdin, stdout, exit } = require('node:process')
    const rl = readline.createInterface({ input: stdin, output: stdout })

    const fourengine = new Fourengine()
    const useBook = await yesNoQuestion(rl, 'Use opening book (y/n)? ')
    if (useBook) {
        await fourengine.enableBook()
        console.log('Opening book is enabled')
    } else {
        console.log('Opening book is disabled')
    }

    while (true) {
        try {
            const variation = await rl.question('Variation: ')
            const position = new Position(variation)
            console.log(position.toString())
            const solution = fourengine.solve(position)
            console.log(solution)
            console.log()
        } catch (err) {
            console.error(err.message)
        }
    }
}

module.exports = {
    BOARD_WIDTH,
    BOARD_HEIGHT,
    Fourengine,
    Position,
}

if (require.main == module) {
    main().catch(e => {
        console.error(e)
    })
}