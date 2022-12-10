const vscode = require('vscode')
const https = require('https')
const childProcess = require('child_process')
const files = require('./files')
const server = require('./server')
const config = require('./config/settings.js')
const messageHandler = require('./messageHandler')

const URL = 'https://dervexhero.github.io/Argon/'
const ITEMS = [
    {
        label: '$(debug-start) Run Argon',
        action: 'run'
    },
    {
        label: '$(debug-stop) Stop Argon',
        action: 'stop'
    },
    {
        label: '$(cloud-download) Update Classes',
        action: 'updateClasses'
    },
    {
        label: '$(settings-gear) Open Argon Settings',
        action: 'openSettings'
    },
    {
        label: '$(run-all) Launch Roblox Studio',
        action: 'launchStudio'
    },
    {
        label: 'Temp',
        action: 'temp'
    }
]

let activated = false
let isRunning = false

function run(autoRun) {
    if (!isRunning) {
        if (vscode.workspace.name) {
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

function updateClasses() {
    if (vscode.workspace.name) {
        files.updateClasses()
    }
    else {
        messageHandler.showMessage('openWorkspace', 1)
    }
}

function launchStudio() {
    childProcess.exec('%LOCALAPPDATA%\\Roblox\\Versions\\RobloxStudioLauncherBeta.exe -ide', function (error) {
        if (error) {
            messageHandler.showMessage('robloxStudioLaunch', 2)
        }
    })
}

function openMenu() {
    let quickPick = vscode.window.createQuickPick()

    quickPick.title = 'Argon'
    quickPick.items = ITEMS

    quickPick.onDidAccept(function() {
        let item = quickPick.selectedItems[0]

        switch (item.action) {
            case 'run':
                run()
                quickPick.dispose()
                break
            case 'stop':
                stop()
                quickPick.dispose()
                break
            case 'updateClasses':
                updateClasses()
                quickPick.dispose()
                break
            case 'openSettings':
                vscode.commands.executeCommand('workbench.action.openSettings', '@ext:dervex.argon')
                quickPick.dispose()
                break
            case 'launchStudio':
                launchStudio()
                quickPick.dispose()
                break
            case 'temp':
                server.debug()
                //require('../.vscode/utils').generateSchema()
                quickPick.dispose()
                break
        }
    })

    quickPick.show()
}

async function activate(context) {
    if (!activated) {
        activated = true

        context.subscriptions.push(vscode.commands.registerCommand('argon.openMenu', openMenu))

        if (config.autoRun) {
            run(true)
        }

        if (config.autoLaunchStudio) {
            launchStudio()
        }

        https.get(URL, (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                if (JSON.parse(body).extension != context.extension.packageJSON.version) {
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
                autoLaunchStudio: extension.get('autoLaunchStudio'),
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