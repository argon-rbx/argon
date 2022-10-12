const vscode = require('vscode')
const path = require('path')
const fs = require('fs')
const https = require('https')
const events = require('./events')
const config = require('../config/settings.js')
const types = require('../config/types.js')
const messageHandler = require('./messageHandler')

const VERSION_URL = 'https://s3.amazonaws.com/setup.roblox.com/versionQTStudio'
const API_URL = 'https://s3.amazonaws.com/setup.roblox.com/$url-API-Dump.json'
const ROOT_NAME = config.rootName

let watchers = []

function verify(parent) {
    if (parent != null && parent != '') {
        return true
    }
}

function getParent(root) {
    if (root.includes(ROOT_NAME) == false) {
        return null
    }

    let dir = root.split('\\')
    let similarName = 0
    let parent = ''

    for (let i = dir.length - 1; i >= 0; i--) {
        if (dir[i].includes('.')) {
            let type = dir[i].split('.')
            type = type[type.length - 1]

            if (events.getTypes().includes(type)) {
                dir[i] = dir[i].replace('.' + type, '')
            }
        }

        if (i != dir.length - 1) {
            parent = dir[i] + '.' + parent
        }
        else {
            parent = dir[i]
        }

        if (dir[i] == ROOT_NAME) {
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

    parent = parent.slice(ROOT_NAME.length + 1)
    return parent
}

function onCreate(uri) {
    uri = path.parse(uri.files[0].fsPath)
    let parent = getParent(uri.dir)

    if (verify(parent) != true) {
        return
    }

    events.create(uri.ext, uri.name, parent)
}

function onSave(uri) {
    let source = uri.getText()
    uri = path.parse(uri.fileName)
    let parent = getParent(uri.dir)

    if (verify(parent) != true) {
        return
    }

    events.update(parent + '.' + uri.name, source)
}

function onDelete(uri) {
    uri = path.parse(uri.files[0].fsPath)
    let parent = getParent(uri.dir)

    if (verify(parent) != true) {
        return
    }

    events.remove(parent + '.' + uri.name)
}

function onRename(uri) {
    uri = uri.files[0]

    let newUri = path.parse(uri.newUri.fsPath)
    let newParent = getParent(newUri.dir)
    
    if (verify(newParent) != true) {
        return
    }

    let oldUri = path.parse(uri.oldUri.fsPath)
    let oldParent = getParent(oldUri.dir)

    if (newUri.name != oldUri.name) {
        if (newUri.ext == '.lua' || newUri.ext == '.luau') {
            let newSplitted = newUri.name.split('.')
            let oldSplitted = oldUri.name.split('.')
            
            if (newSplitted.length != oldSplitted.length) {
                events.changeType(oldParent + '.' + oldUri.name, newSplitted[newSplitted.length - 1], newUri.name)
            }
            else {
                let newName = newSplitted[0]
                let newType = newSplitted[newSplitted.length - 1]
                let oldName = oldSplitted[0]
                let oldType = oldSplitted[newSplitted.length - 1]
                
                if (newName != oldName && newType == oldType) {
                    events.rename(oldParent + '.' + oldUri.name, newUri.name)
                }
                else if (newType != oldType && newName == oldName) {
                    events.changeType(newParent + '.' + newUri.name, newType)
                }
                else {
                    events.changeType(oldParent + '.' + oldUri.name, newSplitted[newSplitted.length - 1], newUri.name)
                }
            }
        }
        else {
            events.rename(oldParent + '.' + oldUri.name, newUri.name)
        }
    }
    else if (newUri.ext != oldUri.ext) {
        events.changeType(newParent + '.' + newUri.name, newUri.ext)
    }
    else {
        events.changeParent(oldParent + '.' + oldUri.name, newParent)
    }
}

function run() {
    let dir = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, 'game')

    if (fs.existsSync(dir) == false) {
        fs.mkdirSync(dir)
    }

    events.setTypes(types)

    if (config.autoUpdate) {
        updateClasses()
    }

    watchers.push(vscode.workspace.onDidCreateFiles(onCreate))
    watchers.push(vscode.workspace.onDidSaveTextDocument(onSave))
    watchers.push(vscode.workspace.onDidDeleteFiles(onDelete))
    watchers.push(vscode.workspace.onDidRenameFiles(onRename))
}

function stop() {
    for (let i = 0; i < watchers.length; i++) {
        watchers[i].dispose()
    }

    watchers.length = 0
}

function updateClasses() {
    function getData(url) {
        https.get(url, (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                try {
                    if (url == VERSION_URL) {
                        getData(API_URL.replace('$url', body))
                    }
                    else {
                        let classes = JSON.parse(body).Classes
                        let newTypes = []

                        for (let i = 0; i < classes.length; i++) {
                            if (classes[i].Tags == undefined) {
                                newTypes.push(classes[i].Name)
                            }
                            else if (classes[i].Tags.includes('NotCreatable') == false) {
                                newTypes.push(classes[i].Name)
                            }
                        }

                        events.setTypes(newTypes)
                        messageHandler.showMessage('databaseUpdated')
                    }
                } catch (error) {
                    messageHandler.showMessage('error', 2)
                }
            })
        
        }).on('error', () => {
            messageHandler.showMessage('serverConnection', 2)
        })
    }

    messageHandler.showMessage('updatingDatabase', 1)
    getData(VERSION_URL)
}

module.exports = {
    run,
    stop,
    updateClasses
}