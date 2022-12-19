const vscode = require('vscode')

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

    json: 'default.project.json',
    separator : '|'
}

if (!settings.compatibilityMode) {
    settings.source = '.source',
    settings.properties = '.properties'
}
else {
    settings.source = 'init',
    settings.properties = 'init.meta'
}

module.exports = settings