const vscode = require('vscode')
const https = require('https')
const childProcess = require('child_process')
const files = require('./files')
const server = require('./server')
const config = require('./config/settings')
const messageHandler = require('./messageHandler')

//@ts-ignore
const winuser = require('./utils/winuser')

const URL = 'https://dervexhero.github.io/Argon/'

let activated = false
let isRunning = false
let func = null

function run(autoRun) {
    if (!isRunning) {
        if (vscode.workspace.name) {
            server.run(function(canConnect) {
                if (canConnect) {
                    files.run()
                    messageHandler.showMessage('argonRunning')
                    isRunning = true
                    if (func) {
                        func()
                    }
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

function launchStudio() {
    childProcess.exec('%LOCALAPPDATA%\\Roblox\\Versions\\RobloxStudioLauncherBeta.exe -ide', function (error) {
        if (error) {
            messageHandler.showMessage('robloxStudioLaunch', 2)
        }
    })
}

function debugPlay() {
    winuser.showStudio(0x74)
}

function debugRun() {
    winuser.showStudio(0x77)
}

function openMenu() {
    let quickPick = vscode.window.createQuickPick()

    quickPick.title = 'Argon' + server.getTitle()
    quickPick.items = [
        {
            label: !isRunning ? '$(debug-start) Run Argon' : '$(debug-stop) Stop Argon',
            detail: !isRunning ? "Run local server and listen for file changes" : "Stop local server and stop listening for file changes",
            action: 'runStop'
        },
        {
            label: '$(settings-gear) Open Argon Settings',
            detail: "Show all available extension settings",
            action: 'openSettings'
        },
        {
            label: '$(breakpoints-view-icon) Start Debugging',
            detail: "Switch to Roblox Studio and start playtest (F5 or F8)",
            action: 'startDebugging'
        },
        {
            label: '$(run-all) Launch Roblox Studio',
            detail: "Open new Roblox Studio instance",
            action: 'launchStudio'
        }
    ]

    quickPick.onDidAccept(function() {
        let item = quickPick.selectedItems[0]

        switch (item.action) {
            case 'runStop':
                !isRunning ? run() : stop()
                quickPick.dispose()
                break
            case 'openSettings':
                vscode.commands.executeCommand('workbench.action.openSettings', '@ext:dervex.argon')
                quickPick.dispose()
                break
            case 'startDebugging':
                let subQuickPick = vscode.window.createQuickPick()
                
                subQuickPick.title = 'Select playtest mode:'
                subQuickPick.items = [
                    {
                        label: '$(vm) Play',
                        description: 'F5',
                        action: 'play'
                    },
                    {
                        label: '$(server) Run',
                        description: 'F8',
                        action: 'run'
                    }
                ]

                subQuickPick.onDidAccept(function() {
                    let subItem = subQuickPick.selectedItems[0]

                    switch (subItem.action) {
                        case 'play':
                            debugPlay()
                            subQuickPick.dispose()
                            break
                        case 'run':
                            debugRun()
                            subQuickPick.dispose()
                            break
                    }
                })

                subQuickPick.show()
                quickPick.dispose()
                break
            case 'launchStudio':
                launchStudio()
                quickPick.dispose()
                break
        }
    })

    func = function() {
        let items = [
            {
                label: !isRunning ? '$(debug-start) Run Argon' : '$(debug-stop) Stop Argon',
                detail: !isRunning ? "Run local server and listen for file changes" : "Stop local server and stop listening for file changes",
                action: 'runStop'
            }
        ]

        // @ts-ignore
        quickPick.items = items.concat(quickPick.items.slice(1))
    }

    quickPick.show()
}

async function activate(context) {
    if (!activated) {
        activated = true

        let menuCommand = vscode.commands.registerCommand('argon.openMenu', openMenu)
        let playCommand = vscode.commands.registerCommand('argon.playDebug', debugPlay)
        let runCommand = vscode.commands.registerCommand('argon.runDebug', debugRun)

        context.subscriptions.push(menuCommand, playCommand, runCommand)

        if (config.autoRun) {
            run(true)
        }

        if (config.autoLaunchStudio && !winuser.isStudioRunning()) {
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
                rootFolder: directories.get('rootFolder'),
                extension: directories.get('extension'),
                compatibilityMode: directories.get('compatibilityMode'),
                autoRun: extension.get('autoRun'),
                autoSetup: extension.get('autoSetup'),
                autoLaunchStudio: extension.get('autoLaunchStudio'),
                hideNotifications: extension.get('hideNotifications'),
                openInPreview: extension.get('openInPreview'),
                debugMode: extension.get('debugMode'),
                host: server.get('host'),
                port: server.get('port'),
            }
            
            if (!settings.compatibilityMode) {
                settings.source = '.source',
                settings.properties = '.properties'
            }
            else {
                settings.source = 'init',
                settings.properties = 'init.meta'
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