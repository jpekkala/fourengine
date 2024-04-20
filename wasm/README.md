# Fourengine

A perfect Connect-4 solver in WebAssembly.

## NodeJS example
```javascript
const { Fourengine, Position } = require('fourengine')

const fourengine = new Fourengine()

const position = new Position('444444')
const solution = fourengine.solve(position)
```

## Opening book
By default, the engine does not use an opening book but it is included in the package. You can enable it like this:
```javascript
const fourengine = new Fourengine()
await fourengine.enableBook()
```
The method is asynchronous because it loads book files from disk.

You can alternatively load the included book files synchronously:
```javascript
const fourengine = new Fourengine()
fourengine.enableBookSync()
```