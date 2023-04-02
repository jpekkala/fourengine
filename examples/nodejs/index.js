const process = require('process')
const { Engine } = require('../../www/pkg')

const variation = process.argv[2] || '444444'

const engine = new Engine()
console.log('Solving variation', variation)
const solution = engine.solve(variation)
console.log('Score is', solution.getScore())
