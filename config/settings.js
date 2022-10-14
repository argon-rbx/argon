const vscode = require('vscode')

const directories = vscode.workspace.getConfiguration('argon.directories')
const extension = vscode.workspace.getConfiguration('argon.extension')
const server = vscode.workspace.getConfiguration('argon.server')

module.exports = {
    rootName: directories.get('rootFolder'),
    separator: directories.get('separator'),
    autoRun: extension.get('autoRun'),
    autoUpdate: extension.get('autoUpdate'),
    host: server.get('host'),
    port: server.get('port')
}