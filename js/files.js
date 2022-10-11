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

function verify(uri, parent) {
    if (uri.dir.includes(ROOT_NAME) && parent != '') {
        return true
    }
}

function getParent(root) {
    let dir = root.split('\\')
    let similarName = 0
    let parent = ''

    for (let i = dir.length - 1; i >= 0; i--) {
        if (dir[i].includes('.')) {
            let separated = dir[i].split('.')
            let type = separated[separated.length - 1]

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

    if (verify(uri, parent) != true) {
        return
    }

    events.onCreate(uri.ext, uri.name, parent)
}

function onSave(uri) {
    let source = uri.getText()
    uri = path.parse(uri.fileName)
    let parent = getParent(uri.dir)

    if (verify(uri, parent) != true) {
        return
    }

    events.onSave(parent + '.' + uri.name, source)
}

function onDelete(uri) {
    uri = path.parse(uri.files[0].fsPath)
    let parent = getParent(uri.dir)

    if (verify(uri, parent) != true) {
        return
    }

    events.onDelete(parent + '.' + uri.name)
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
}

function stop() {
    for (let i = 0; i < watchers.length; i++) {
        watchers[i].dispose()
    }

    watchers.length = 0
}

module.exports = {
    run,
    stop,
    updateClasses
}