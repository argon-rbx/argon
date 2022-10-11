const http = require('http')
const config = require('../config/settings.js')
const requests = require('./requests')

const PORT = config.port
const HOST = config.host

let localServer = http.createServer(requestListener)

function requestListener(request, response) {
    let headers = request.headers
    let data = null;

    switch (headers.action) {
        case 'getSync':
            data = requests.getSync()
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