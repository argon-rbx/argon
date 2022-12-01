const http = require('http')
const vscode = require('vscode')
const path = require('path')
const ffi = require('ffi-napi')
const config = require('../config/settings.js')
const website = require('../config/website.js')
const events = require('./events')
const files = require('./files')
const twoWaySync = require('./twoWaySync')

const URL = 'http://$host:$port/'
const user32 = new ffi.Library('user32', {
    'GetForegroundWindow': ['long', []],
    'ShowWindow': ['bool', ['long', 'int']],
    'SetForegroundWindow': ['bool', ['long']],

    'GetAsyncKeyState': ['short', ['int']],
    'keybd_event': ['void', ['long', 'long', 'long', 'long']]
})

let server = http.createServer(requestListener)
let lastSync = Date.now()
let sockets = new Set() //Temp fix for forcing server to stop (until Electron adds support for node.js 18.2.0+)
let isConnected = false
let requestsLeft = 0
let chunks = []
let window = user32.GetForegroundWindow()

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
        case 'setSync':
            var body = ''
        
            request.on('data', (chunk) => {
                body += chunk
            })

            request.on('end', () => {
                twoWaySync.sync(body)
            })
            break
        case 'openFile':
            var body = ''
        
            request.on('data', (chunk) => {
                body += chunk
            })
    
            request.on('end', () => {
                openFile(body)
            })
            break
        case 'init':
            if (Date.now() - lastSync > 500) {
                isConnected = false
            }

            data = JSON.stringify(isConnected)

            if (isConnected == false) {
                lastSync = Date.now()
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
            events.queue.length = 0
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

async function openFile(file) {
    file = path.join(files.getRootDir(), JSON.parse(file) + config.extension) //TODO: .source file detection
    file = await vscode.workspace.openTextDocument(file)
    await vscode.window.showTextDocument(file, {preview: config.openInPreview})

    let pressed = false

    if ((user32.GetAsyncKeyState(0x12) & 0x8000) == 0) {
        pressed = true
        user32.keybd_event(0x12, 0, 0x0001 | 0, 0)
    }

    user32.ShowWindow(window, 9)
    user32.SetForegroundWindow(window)

    if (pressed) {
        user32.keybd_event(0x12, 0, 0x0001 | 0x0002, 0)
    }
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

if (vscode.window.state.focused) {
    window = user32.GetForegroundWindow()
}
else {
    let watcher = vscode.window.onDidChangeWindowState((state) => {
        if (state.focused) {
            window = user32.GetForegroundWindow()
            watcher.dispose()
        }
    })
}