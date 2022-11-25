const vscode = require('vscode')

const directories = vscode.workspace.getConfiguration('argon.directories')
const extension = vscode.workspace.getConfiguration('argon.extension')
const server = vscode.workspace.getConfiguration('argon.server')

module.exports = {
    rootName: directories.get('rootFolder'),
    extension: directories.get('extension'),
    autoRun: extension.get('autoRun'),
    autoUpdate: extension.get('autoUpdate'),
    autoCreateFolder: extension.get('autoCreateFolder'),
    hideNotifications: extension.get('hideNotifications'),
    host: server.get('host'),
    port: server.get('port')
}