const http = require('http')
const config = require('../config/settings.js')
const events = require('./events')
const files = require('./files')

const PORT = config.port
const HOST = config.host

let localServer = http.createServer(requestListener)

function requestListener(request, response) {
    let headers = request.headers
    let newData = null;

    switch (headers.action) {
        case 'getSync':
            newData = JSON.stringify(events.queue)
            events.queue.length = 0
            break
        case 'port':
            let body = ''
        
            request.on('data', (data) => {
                body += data
            })
    
            request.on('end', () => {
                files.port(body)
            })
            break
    }

    response.writeHead(200)
    response.end(newData)
}

function run() {
    localServer.listen(PORT, HOST);
}

function stop() {
    localServer.close()
}

module.exports = {
    run,
    stop
}