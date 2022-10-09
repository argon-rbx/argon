const vscode = require('vscode')
const fs = require('fs')
const path = require('path')
const events = require('./events')

let dir = null
let watcher = null

function onCreate(uri) {
    uri = path.parse(uri.fsPath)
    events.onCreate(uri.name, uri.ext)
}

function onChange() {
    events.onChange()
}

function onDelete() {
    events.onDelete()
}

function verify() {
    dir = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, 'game')

    if (fs.existsSync(dir) == false) {
        fs.mkdirSync(dir)
    }
}

function run() {
    verify()

    watcher = vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(dir, '*'))
    watcher.onDidCreate(onCreate)
    watcher.onDidChange(onChange)
    watcher.onDidDelete(onDelete)
}

function stop() {
    watcher.dispose()
}

module.exports = {
    run,
    stop
}