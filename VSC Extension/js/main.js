const vscode = require('vscode')
const https = require('https')
const files = require('./files')
const server = require('./server')
const config = require('../config/settings.js')
const messageHandler = require('./messageHandler')

const URL = 'https://dervexhero.github.io/Argon/'

let activated = false
let isRunning = false

function run() {
    if (isRunning == false) {
        if (vscode.workspace.name !== undefined) {
            server.run(function(canConnect) {
                if (canConnect) {
                    files.run()
                    messageHandler.showMessage('argonRunning')
                    isRunning = true
                }
                else {
                    messageHandler.showMessage('alreadyRunning', 2)
                }
            })

        }
        else {
            messageHandler.showMessage('openWorkspace', 1)
        }
    }
    else {
        messageHandler.showMessage('alreadyRunning', 1)
    }
}

function stop() {
    if (isRunning) {
        messageHandler.showMessage('argonStopped')
        files.stop()
        server.stop()
        isRunning = false
    }
    else {
        messageHandler.showMessage('notRunning', 1)
    }
}

function update() {
    if (isRunning) {
        files.updateClasses()
    }
    else {
        messageHandler.showMessage('notRunning', 1)
    }
}

async function activate(context) {
    if (activated == false) {
        activated = true

        let runCommand = vscode.commands.registerCommand('argon.run', run)
        let stopCommand = vscode.commands.registerCommand('argon.stop', stop)
        let updateCommand = vscode.commands.registerCommand('argon.update', update)
    
        context.subscriptions.push(runCommand, stopCommand, updateCommand)
    
        if (config.autoRun) {
            run()
        }

        https.get(URL, (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                if (body != context.extension.packageJSON.version) {
                    messageHandler.showMessage('outdatedVersion', 1)
                }
            })
        })
    }
}

module.exports = {
	activate
}