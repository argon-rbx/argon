const vscode = require('vscode')
const https = require('https')
const childProcess = require('child_process')
const files = require('./files')
const server = require('./server')
const config = require('./config/settings.js')
const messageHandler = require('./messageHandler')

//@ts-ignore
const winuser = require('./utils/winuser')

const URL = 'https://dervexhero.github.io/Argon/'
const ITEMS = [
    {
        label: '$(debug-start) Run Argon',
        detail: "Run local server and listen for file changes",
        action: 'run'
    },
    {
        label: '$(debug-stop) Stop Argon',
        detail: "Stop local server and stop listening for file changes",
        action: 'stop'
    },
    {
        label: '$(settings-gear) Open Argon Settings',
        detail: "Show all available extension settings",
        action: 'openSettings'
    },
    {
        label: '$(cloud-download) Update Classes',
        detail: "Download latest Roblox API",
        action: 'updateClasses'
    },
    {
        label: '$(breakpoints-view-icon) Start Debugging',
        detail: "Switch to Roblox Studio and start playtest (F5)",
        action: 'startDebugging'
    },
    {
        label: '$(run-all) Launch Roblox Studio',
        detail: "Open new Roblox Studio instance",
        action: 'launchStudio'
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

function debug() {
    switch (config.debugMode) {
        case "Play":
            winuser.showStudio(0x74)
            break;
        case "Run":
            winuser.showStudio(0x77)
            break
    }
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
            case 'openSettings':
                vscode.commands.executeCommand('workbench.action.openSettings', '@ext:dervex.argon')
                quickPick.dispose()
                break
            case 'updateClasses':
                updateClasses()
                quickPick.dispose()
                break
            case 'startDebugging':
                debug()
                quickPick.dispose()
                break
            case 'launchStudio':
                launchStudio()
                quickPick.dispose()
                break
        }
    })

    quickPick.show()
}

async function activate(context) {
    if (!activated) {
        activated = true

        let menuCommand = vscode.commands.registerCommand('argon.openMenu', openMenu)
        let debugCommand = vscode.commands.registerCommand('argon.startDebugging', debug)

        context.subscriptions.push(menuCommand, debugCommand)

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
                debugMode: extension.get('debugMode'),
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