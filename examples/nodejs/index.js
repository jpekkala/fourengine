const process = require('process')
// From local build:
const { Engine } = require('../../www/nodejs')
// From npm:
// const { Engine } = require('fourengine/nodejs')

const variation = process.argv[2] || '444444'

const engine = new Engine()
console.log('Solving variation', variation)
const solution = engine.solve(variation)
console.log('Score is', solution.getScore())
