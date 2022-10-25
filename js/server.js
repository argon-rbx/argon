const http = require('http')
const config = require('../config/settings.js')
const events = require('./events')
const files = require('./files')

const PORT = config.port
const HOST = config.host

let localServer = http.createServer(requestListener)
let requestsLeft = 0
let chunks = []

function requestListener(request, response) {
    let headers = request.headers
    let data = null;

    switch (headers.action) {
        case 'getSync':
            data = JSON.stringify(events.queue)
            events.queue.length = 0
            break
        case 'getState':
            data = JSON.stringify(Date.now() - files.getUnix())
            break
        case 'portInstances':
            var body = ''
        
            request.on('data', (chunk) => {
                body += chunk
            })
    
            request.on('end', () => {
                files.portInstances(body)
            })
            break
        case 'portScripts':
            var body = ''
        
            request.on('data', (chunk) => {
                body += chunk
            })
    
            request.on('end', () => {
                files.portScripts(body)
            })
            break
        case 'portProject':
            chunks = files.portProject()
            requestsLeft = chunks.length
            data = JSON.stringify({Project: events.queue, Length: requestsLeft})
            events.queue.length = 0
            break
        case 'portProjectSource':
            data = JSON.stringify({Chunk: chunks[chunks.length - requestsLeft], Length: requestsLeft - 1})
            requestsLeft--

            if (requestsLeft <= 0) {
                requestsLeft = 0
                chunks.length = 0
            }
            break
    }

    response.writeHead(200)
    response.end(data)
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