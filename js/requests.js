const events = require('./events')

function getSync() {
    let json = JSON.stringify(events.queue)
    events.queue.length = 0
    return json
}

module.exports = {
    getSync
}