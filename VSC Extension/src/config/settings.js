const vscode = require('vscode')
const os = require('os')

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

    source: null,
    properties: null,
    separator : '|',
    os: os.platform(),
    osSeparator: os.platform() == 'win32' ? '\\' : '/'
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

module.exports = settings