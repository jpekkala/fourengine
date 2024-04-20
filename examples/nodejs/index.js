const process = require('process')
// From local build:
const { Fourengine, Position } = require('../../wasm')
// From npm:
// const { Fourengine, Position } = require('fourengine')

const position = new Position(process.argv[2] || '444444')
console.log('Solving position:')
console.log(position.toString())

const fourengine = new Fourengine()
const solution = fourengine.solve(position)
console.log('Score is', solution.score)
