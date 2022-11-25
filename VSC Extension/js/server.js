const http = require('http')
const config = require('../config/settings.js')
const website = require('../config/website.js')
const events = require('./events')
const files = require('./files')

const URL = 'http://$host:$port/'

let server = http.createServer(requestListener)
let lastSync = Date.now()
let sockets = new Set() //Temp fix for forcing server to stop (until Electron adds support for node.js 18.2.0+)
let isConnected = false
let requestsLeft = 0
let chunks = []

let syncCount = 5
let uptime = 0

function getTime() {
    let time = Date.now() - uptime
    let hours, minutes, seconds

    hours = Math.floor(time / 1000 / 60 / 60)
    minutes = Math.floor((time / 1000 / 60 / 60 - hours) * 60)
    seconds = Math.floor(((time / 1000 / 60 / 60 - hours) * 60 - minutes) * 60)

    hours < 10 ? hours = `0${hours}`: hours = `${hours}`
    minutes < 10 ? minutes = `0${minutes}`: minutes = `${minutes}`
    seconds < 10 ? seconds = `0${seconds}`: seconds = `${seconds}`

    return hours + ':' + minutes + ':' + seconds
}

function requestListener(request, response) {
    let headers = request.headers
    let data = null;

    switch (headers.action) {
        case 'getSync':
            data = JSON.stringify(events.queue)

            if (events.queue.length > 0) {
                events.queue.length = 0
                syncCount ++
            }

            lastSync = Date.now()
            break
        case 'init':
            if (Date.now() - lastSync > 500) {
                isConnected = false
            }

            data = JSON.stringify(isConnected)

            if (isConnected == false) {
                isConnected = true
            }
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
        default:
            data = website.replace('$time', getTime()).replace('$synces', syncCount.toString())
            break
    }

    response.writeHead(200)
    response.end(data)
}

function run(callback) {
    let canConnect = true

    http.get(URL.replace('$host', config.host).replace('$port', config.port), () => {
        canConnect = false
    })

    setTimeout(() => {
        if (canConnect) {
            server.listen(config.port, config.host);
            uptime = Date.now()
        }
        callback(canConnect)
    }, 100)
}

function stop() {
    for (let socket of sockets) {
        socket.destroy();
        sockets.delete(socket);
    }

    server.close()
    syncCount = 0
}

module.exports = {
    run,
    stop
}

server.on('connection', (socket) => {
    sockets.add(socket);

    server.once('close', () => {
        sockets.delete(socket);
    })
})