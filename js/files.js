const vscode = require('vscode')
const fs = require('fs')
const path = require('path')
const events = require('./events')

let watchers = []

function getParent(root) {
    let dir = root.split('\\')
    let similarName = 0
    let parent = ''

    for (let i = dir.length - 1; i >= 0; i--) {
        if (i != dir.length - 1) {
            parent = dir[i] + '.' + parent
        }
        else {
            parent = dir[i]
        }

        if (dir[i] == vscode.workspace.name) {
            let len = root.split(dir[i]).length - 1

            if (len > 1 && similarName == 0)  {
                similarName = len - 1
            }
            else if (similarName > 1) {
                similarName--
            }
            else {
                break
            }
        }
    }

    parent = parent.slice(vscode.workspace.name.length + 1)
    return parent
}

function onCreate(uri) {
    uri = path.parse(uri.files[0].fsPath)
    events.onCreate(uri.ext, uri.name, getParent(uri.dir))
}

function onSave(uri) {
    let source = uri.getText()
    uri = path.parse(uri.fileName)

    events.onSave(getParent(uri.dir) + '.' + uri.name, source)
}

function onDelete(uri) {
    uri = path.parse(uri.files[0].fsPath)
    events.onDelete(getParent(uri.dir) + '.' + uri.name)
}

function run() {
    watchers.push(vscode.workspace.onDidCreateFiles(onCreate))
    watchers.push(vscode.workspace.onDidSaveTextDocument(onSave))
    watchers.push(vscode.workspace.onDidDeleteFiles(onDelete))
}

function stop() {
    for (let i = 0; i < watchers.length; i++) {
        watchers[i].dispose()
    }

    watchers.length = 0
}

module.exports = {
    run,
    stop
}