const http = require('http')
const vscode = require('vscode')
const path = require('path')
const fs = require('fs')
const childProcess = require('child_process')
const config = require('./config/settings')
const events = require('./events')
const files = require('./files')
const twoWaySync = require('./twoWaySync')
const apiDump = require('./config/apiDump')
const messageHandler = require('./messageHandler')

if (config.os == 'win32') {
    // @ts-ignore
    var winuser = require('./utils/winuser')
}

const URL = 'http://$host:$port/'

let server = http.createServer(requestListener)
let statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, -69420)
let sockets = new Set()
let isConnected = false
let isRunning = false
let requestsLeft = 0
let version = null
let chunks = []
let title = ''

let uptime = 0
let linesSynced = -1
let filesSynced = -1
let projectsPorted = 0
let sessionsStarted = 0

let stats = {
    hoursUsed: 0,
    linesSynced: 0,
    filesSynced: 0,
    projectsPorted: 0,
    sessionsStarted: 0,
}

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

function updateStats() {
    stats.hoursUsed += uptime == 0 ? 0 : Math.round((Date.now() - uptime) / 1000 / 60 / 60)
    stats.linesSynced += linesSynced
    stats.filesSynced += filesSynced
    stats.projectsPorted += projectsPorted
    stats.sessionsStarted += sessionsStarted

    return stats
}

function resetStats() {
    updateStats()

    uptime = 0
    linesSynced = 0
    filesSynced = 0
    projectsPorted = 0
    sessionsStarted = 0
}

function updateStatusBar() {
    statusBarItem.text = isRunning ? isConnected ? '$(pass-filled) Argon' : '$(pass) Argon' : '$(stop) Argon'
    statusBarItem.command = 'argon.openMenu'
    statusBarItem.name = 'Argon'

    statusBarItem.tooltip = 'Running on: ' + (isRunning ? config.host + ':' + config.port : 'NONE')
    + '\nConnected to: ' + (title || 'NONE')
    + '\nServer uptime: ' + getUptime()
    + '\nLines synced: ' + Math.max(0, linesSynced)
    + '\nFiles synced: ' + Math.max(0, filesSynced)
}

async function countLines(data) {
    linesSynced += data.split('\\n').length
}

function requestListener(request, response) {
    let headers = request.headers
    let data = null;

    switch (headers.action) {
        case 'getSync':
            data = JSON.stringify(events.queue)

            if (events.queue.length > 0) {
                events.queue.length = 0
                filesSynced++
                countLines(data)
                updateStatusBar()
            }

            if (!isConnected) {
                isConnected = true
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
            data = JSON.stringify({
                State: isConnected,
                Title: files.getTitle(),
                Version: version,
                Separator: config.osSeparator
            })
            
            if (!isConnected) {
                isConnected = true
                sessionsStarted++

                if (config.os == 'win32') {
                    winuser.resetWindow()
                }

                updateStatusBar()
            }

            break
        case 'disconnect':
            isConnected = false
            title = ''
            updateStatusBar()
            break
        case 'syncTitle':
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

            projectsPorted++
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
            projectsPorted++
            break
        case 'portProjectSource':
            data = JSON.stringify({Chunk: chunks[chunks.length - requestsLeft], Length: requestsLeft - 1})
            requestsLeft--

            if (requestsLeft <= 0) {
                requestsLeft = 0
                chunks.length = 0
            }
            break
        case 'clearFolders':
            files.clearFolders()
            break
        case 'getApiDump':
            data = JSON.stringify(apiDump)
            break
        default:
            let website = fs.readFileSync(path.resolve(__dirname, './config/website.html')).toString()
            data = website.replace('$uptime', getUptime())
            .replace('$linesSynced', linesSynced.toString())
            .replace('$filesSynced', filesSynced.toString())
            .replace('$projectsPorted', projectsPorted.toString())
            .replace('$sessionsStarted', sessionsStarted.toString())
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
            isRunning = true
            uptime = Date.now()

            events.syncTitle(files.getTitle())
        }

        updateStatusBar()
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
    isRunning = false
    title = ''

    resetStats()
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

        file.File += config.osSeparator + suffix
    }

    file = files.applyCustomPaths(path.join(files.getRootDir(), file.File + config.extension))

    vscode.workspace.openTextDocument(file).then(file => {
        vscode.window.showTextDocument(file, {preview: config.openInPreview}).then(() => {
            if (line != 1) {
                let range = vscode.window.activeTextEditor.document.lineAt(line - 1).range
                let selection = new vscode.Selection(range.start, range.end)
                vscode.window.activeTextEditor.selection = selection
                vscode.window.activeTextEditor.revealRange(range)
            }

            events.closeFile()

            switch (config.os) {
                case 'win32':
                    winuser.showVSC(vscode.workspace.name)
                    break
                case 'darwin':
                    //TODO
                    console.log(1);
                    childProcess.exec('open -a Visual\ Studio\ Code')
                    break
                default:
                    messageHandler.show('unsupportedOS', 2)
                    break
            }
        })
    }).then(undefined, () => {})
}

function setVersion(ver) {
    version = ver
}

function getTitle() {
    return title
}

module.exports = {
    updateStats,
    run,
    stop,
    setVersion,
    getTitle
}

statusBarItem.show()

server.on('connection', (socket) => {
    sockets.add(socket);

    socket.on('close', () => {
        sockets.delete(socket)
        isConnected = false
        title = ''
        updateStatusBar()
    })
})