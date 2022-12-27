const http = require('http')
const vscode = require('vscode')
const path = require('path')
const fs = require('fs')
const config = require('./config/settings')
const events = require('./events')
const files = require('./files')
const twoWaySync = require('./twoWaySync')
const apiDump = require('./config/apiDump')

//@ts-ignore
const winuser = require('./utils/winuser')

const URL = 'http://$host:$port/'

let server = http.createServer(requestListener)
let statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, -69420)
let sockets = new Set() //Temp fix for forcing server to stop (until Electron adds support for node.js 18.2.0+)
let isConnected = false
let requestsLeft = 0
let chunks = []

let syncCount = 0
let uptime = 0
let title = ''

function getUptime() {
    if (uptime == 0) {
        return '00:00:00'
    }

    let time = Date.now() - uptime
    let hours, minutes, seconds

    hours = Math.floor(time / 1000 / 60 / 60)
    minutes = Math.floor((time / 1000 / 60 / 60 - hours) * 60)
    seconds = Math.floor(((time / 1000 / 60 / 60 - hours) * 60 - minutes) * 60)

    hours < 10 ? hours = `0${hours}` : hours = `${hours}`
    minutes < 10 ? minutes = `0${minutes}` : minutes = `${minutes}`
    seconds < 10 ? seconds = `0${seconds}` : seconds = `${seconds}`

    return hours + ':' + minutes + ':' + seconds
}

function updateStatusBar() {
    statusBarItem.text = !isConnected ? '$(circle-large-outline) Argon' : '$(pass-filled) Argon'
    statusBarItem.tooltip = 'Connected to: ' + title.replace(' - ', '') + '\nServer uptime: ' + getUptime() + '\nSync count: ' + syncCount
    statusBarItem.command = 'argon.openMenu'
    statusBarItem.name = 'Argon'
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
                updateStatusBar()   
            }
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
            title = files.getTitle()
            data = JSON.stringify({
                State: isConnected,
                Title: title
            })

            if (!isConnected) {
                isConnected = true
                winuser.resetWindow()
                updateStatusBar()
            }

            break
        case 'disconnect':
            isConnected = false
            title = ''
            updateStatusBar()
            break
        case 'setTitle':
            var body = ''

            request.on('data', (chunk) => {
                body += chunk
            })

            request.on('end', () => {
                title = body
                updateStatusBar()
            })
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
        case 'portProperties':
            var body = ''

            request.on('data', (chunk) => {
                body += chunk
            })

            request.on('end', () => {
                files.portProperties(body)
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
        case 'getApiDump':
            data = JSON.stringify(apiDump)
            break
        default:
            let website = fs.readFileSync(path.resolve(__dirname, './config/website.html')).toString()
            data = website.replace('$time', getUptime()).replace('$synces', syncCount.toString())
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
    isConnected = false
    syncCount = 0
    uptime = 0
    title = ''

    updateStatusBar()
}

function openFile(file) {
    file = JSON.parse(file)
    let suffix = file.Type
    let line = file.Line

    if (suffix) {
        switch (suffix) {
            case 'Script':
                suffix = config.source + '.server'
                break
            case 'LocalScript':
                suffix = config.source + '.client'
                break
            case 'ModuleScript':
                suffix = config.source
                break
        }

        file.File += '\\' + suffix
    }

    file = path.join(files.getRootDir(), file.File + config.extension)

    vscode.workspace.openTextDocument(file).then(file => {
        vscode.window.showTextDocument(file, {preview: config.openInPreview}).then(() => {
            if (line != 1) {
                let range = vscode.window.activeTextEditor.document.lineAt(line - 1).range
                let selection = new vscode.Selection(range.start, range.end)
                vscode.window.activeTextEditor.selection = selection
                vscode.window.activeTextEditor.revealRange(range)
            }

            events.queue.push({Action: 'closeFile'})
            winuser.showVSC(vscode.workspace.name)
        })
    }).then(undefined, () => {})
}

function getTitle() {
    return title
}

module.exports = {
    run,
    stop,
    getTitle
}

statusBarItem.show()

server.on('connection', (socket) => {
    sockets.add(socket);

    server.once('close', () => {
        sockets.delete(socket);
    })
})