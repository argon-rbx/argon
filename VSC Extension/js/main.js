const vscode = require('vscode')
const https = require('https')
const childProcess = require('child_process')
const files = require('./files')
const server = require('./server')
const config = require('../config/settings.js')
const messageHandler = require('./messageHandler')

const URL = 'https://dervexhero.github.io/Argon/'

let activated = false
let isRunning = false

function run(autoRun) {
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
    else if (autoRun != false) {
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
    if (vscode.workspace.name !== undefined) {
        files.updateClasses()
    }
    else {
        messageHandler.showMessage('openWorkspace', 1)
    }
}

function launchRoblox() {
    childProcess.exec('%LOCALAPPDATA%\\Roblox\\Versions\\RobloxStudioLauncherBeta.exe', function (error) {
        if (error) {
            messageHandler.showMessage('robloxStudioLaunch', 2)
        }
    })
}

async function activate(context) {
    if (activated == false) {
        activated = true

        let runCommand = vscode.commands.registerCommand('argon.run', run)
        let stopCommand = vscode.commands.registerCommand('argon.stop', stop)
        let updateCommand = vscode.commands.registerCommand('argon.update', update)
        let launchRobloxCommand = vscode.commands.registerCommand('argon.launchRoblox', launchRoblox)

        context.subscriptions.push(runCommand, stopCommand, updateCommand, launchRobloxCommand)

        if (config.autoRun) {
            run(true)
        }

        https.get(URL, (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                if (JSON.parse(body).plugin != context.extension.packageJSON.version) {
                    messageHandler.showMessage('outdatedVersion', 1)
                }
            })
        })

        vscode.workspace.onDidChangeConfiguration(function() {
            const directories = vscode.workspace.getConfiguration('argon.directories')
            const extension = vscode.workspace.getConfiguration('argon.extension')
            const server = vscode.workspace.getConfiguration('argon.server')
        
            let settings = {
                rootName: directories.get('rootFolder'),
                extension: directories.get('extension'),
                autoRun: extension.get('autoRun'),
                autoUpdate: extension.get('autoUpdate'),
                autoCreateFolder: extension.get('autoCreateFolder'),
                hideNotifications: extension.get('hideNotifications'),
                openInPreview: extension.get('openInPreview'),
                host: server.get('host'),
                port: server.get('port')
            }

            for (let key in settings) {
                config[key] = settings[key]
            }
        })
    }
}

module.exports = {
	activate
}