const vscode = require('vscode')
const files = require('./files')
const server = require('./server')
const messageHandler = require('./messageHandler')

let isRunning = false

function run() {
    if (isRunning == false) {
        if (vscode.workspace.name !== undefined) {
            messageHandler.showMessage('argonRunning')
            files.run()
            server.run()
            isRunning = true
        }
        else {
            messageHandler.showMessage('openWorkspace', 1)
        }
    }
}

function stop() {
    if (isRunning) {
        messageHandler.showMessage('argonStopped')
        files.stop()
        server.stop()
        isRunning = false
    }
}

function update() {
    if (isRunning) {
        files.updateClasses()
    }
}

module.exports = {
    run,
    stop,
    update
}