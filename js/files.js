const vscode = require('vscode')
const fs = require('fs')
const path = require('path')
const events = require('./events')

let watcher = null

function onCreate(uri) {
    uri = path.parse(uri.files[0].fsPath)

    let root = uri.dir
    let dir = root.split('\\')
    let sameName = 0
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

            if (len > 1 && sameName == 0)  {
                sameName = len - 1
            }
            else if (sameName > 1) {
                sameName--
            }
            else {
                break
            }
        }
    }

    parent = parent.slice(vscode.workspace.name.length + 1)

    events.onCreate(uri.ext, uri.name, parent)
}

function onChange() {
    events.onChange()
}

function onDelete() {
    events.onDelete()
}

function run() {
    vscode.workspace.onDidCreateFiles(onCreate)
}

function stop() {
    watcher.dispose()
}

module.exports = {
    run,
    stop
}