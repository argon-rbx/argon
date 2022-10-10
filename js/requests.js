const events = require('./events')

function getSync() {
    console.log(events.queue);
    return JSON.stringify(events.queue)
}

module.exports = {
    getSync
}