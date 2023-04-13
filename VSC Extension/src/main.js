// @ts-nocheck
const vscode = require('vscode')
const https = require('https')
const path = require('path')
const fs = require('fs')
const os = require('os')
const childProcess = require('child_process')
const files = require('./files')
const events = require('./events')
const server = require('./server')
const config = require('./config/settings')
const messageHandler = require('./messageHandler')
const apiUpdater = require('./utils/apiUpdater')

if (config.os == 'win32') {
    var winuser = require('./utils/winuser')
}

const APPLE_SCRIPT = `osascript -e 'tell application "System Events" to return name of processes whose background only is false'`
const VERSION_URL = 'https://dervexhero.github.io/Argon/'
const API_OPTIONS = {
    hostname: 'argonstatsapi.web.app',
    method: 'POST',
    path: '/update',
    headers: {'Content-Type': 'application/json'}
}

let extensionMode = null
let isRunning = false
let func = null

function run(autoRun) {
    if (!isRunning) {
        if (vscode.workspace.name) {
            server.run(function(canConnect) {
                if (canConnect) {
                    files.run()
                    messageHandler.show('argonRunning')
                    isRunning = true
                    if (func) {
                        func()
                    }
                }
                else {
                    messageHandler.show('alreadyRunning', 2)
                }
            })

        }
        else {
            messageHandler.show('openWorkspace', 1)
        }
    }
    else if (autoRun != false) {
        messageHandler.show('alreadyRunning', 1)
    }
}

function stop() {
    if (isRunning) {
        messageHandler.show('argonStopped')
        files.stop()
        server.stop()
        isRunning = false
    }
    else {
        messageHandler.show('notRunning', 1)
    }
}

function executeSnippet() {
    let selection = vscode.window.activeTextEditor.document.getText(vscode.window.activeTextEditor.selection)

    if (selection) {
        events.executeSnippet(selection)
    }
    else {
        events.executeSnippet(vscode.window.activeTextEditor.document.getText())
    }

    if (config.snippetExecutionMode) {
        switch (config.os) {
            case 'win32':
                winuser.showStudio()
                break
            case 'darwin':
                childProcess.exec('open -a RobloxStudio')
                break
            default:
                messageHandler.show('unsupportedOS', 2)
                break
        }
    }
}

function launchStudio() {
    switch (config.os) {
        case 'win32':
            childProcess.exec('%LOCALAPPDATA%\\Roblox\\Versions\\RobloxStudioLauncherBeta.exe -ide', function(error) {
                if (error) {
                    messageHandler.show('robloxStudioLaunch', 2)
                }
            })

            break
        case 'darwin':
            childProcess.exec('open -a RobloxStudio', function(error) {
                if (error) {
                    messageHandler.show('robloxStudioLaunch', 2)
                }
            })

            break
        default:
            messageHandler.show('unsupportedOS', 2)
            break
    }
}

function debugPlay() {
    //winuser.showStudio(0x74)
}

function debugRun() {
    //winuser.showStudio(0x77)
}

function debugStart() {
    //winuser.showStudio(0x76)
}

function openMenu() {
    let quickPick = vscode.window.createQuickPick()

    quickPick.title = 'Argon' + (server.getTitle() ? ' - ' + server.getTitle() : '')
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
            label: '$(layout-panel) Execute Snippet',
            detail: "Execute selected snippet in Roblox Studio's commad bar (F6)",
            action: 'executeSnippet'
        },
        {
            label: '$(breakpoints-view-icon) Start Debugging',
            detail: "Switch to Roblox Studio and start playtest in selected mode (F5, F8, F7)",
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
            case 'executeSnippet':
                executeSnippet()
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
                        label: '$(server-environment) Run',
                        description: 'F8',
                        action: 'run'
                    },
                    {
                        label: '$(server) Start',
                        description: 'F7',
                        action: 'start'
                    },
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
                        case 'start':
                            debugStart()
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

        quickPick.items = items.concat(quickPick.items.slice(1))
    }

    quickPick.show()
}

function removeStudioShortcut() {
    if (config.removeStudioShortcut) {
        let shortcut = path.join(os.homedir(), 'Desktop\\Roblox Studio.lnk')

        if (fs.existsSync(shortcut)) {
            fs.rmSync(shortcut)
        }
    }
}

async function activate(context) {
    let menuCommand = vscode.commands.registerCommand('argon.openMenu', openMenu)
    let playCommand = vscode.commands.registerCommand('argon.playDebug', debugPlay)
    let runCommand = vscode.commands.registerCommand('argon.runDebug', debugRun)
    let startCommand = vscode.commands.registerCommand('argon.startDebug', debugStart)
    let executeCommand = vscode.commands.registerCommand('argon.executeSnippet', executeSnippet)

    context.subscriptions.push(menuCommand, playCommand, runCommand, startCommand, executeCommand)
    server.setVersion(context.extension.packageJSON.version)
    extensionMode = context.extensionMode

    if (config.autoRun) {
        run(true)
    }

    //TEMP
    setTimeout(() => {
        console.log(1);
        childProcess.exec('open -a Visual\ Studio\ Code') 
    }, 1000);

    if (config.autoLaunchStudio) {
        switch (config.os) {
            case 'win32':
                if (!winuser.isStudioRunning()) {
                    launchStudio()
                }
                break
            case 'darwin':
                childProcess.exec(APPLE_SCRIPT).stdout.on('data', function(data) {
                    if (!data.includes('RobloxStudio')) {
                        launchStudio()
                    }
                })
                break
            default:
                messageHandler.show('unsupportedOS', 2)
                break
        }
    }

    removeStudioShortcut()
    apiUpdater.generateApiDump(context.extensionPath)

    https.get(VERSION_URL, (response) => {
        let body = ''
    
        response.on('data', (data) => {
            body += data
        })

        response.on('end', () => {
            if (JSON.parse(body).version != context.extension.packageJSON.version) {
                messageHandler.show('outdatedVersion', 1)
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
            projectFile: directories.get('projectFile'),
            compatibilityMode: directories.get('compatibilityMode'),

            autoRun: extension.get('autoRun'),
            autoSetup: extension.get('autoSetup'),
            autoLaunchStudio: extension.get('autoLaunchStudio'),
            removeStudioShortcut: extension.get('removeStudioShortcut'),
            hideNotifications: extension.get('hideNotifications'),
            openInPreview: extension.get('openInPreview'),
            snippetExecutionMode: extension.get('snippetExecutionMode'),

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

            if (settings.projectFile == '.argon') {
                settings.projectFile == 'default'
            }
        }

        for (let key in settings) {
            config[key] = settings[key]
        }
    })
}

async function deactivate() {
    if (extensionMode == 2) {
        return
    }

    let promise = new Promise(function(resolve) {
        let request = https.request(API_OPTIONS, (response) => {
            response.on('end', () => {
                resolve()
            })

            response.on('error', () => {
                resolve()
            })
        })

        request.write(JSON.stringify(server.updateStats()))
        request.end()

        removeStudioShortcut()
    })

    return promise
}

module.exports = {
	activate,
    deactivate
}