const vscode = require('vscode')
const path = require('path')
const fs = require('fs')
const https = require('https')
const events = require('./events')
const config = require('../config/settings.js')
const types = require('../config/types.js')
const messageHandler = require('./messageHandler')

const VERSION_URL = 'https://s3.amazonaws.com/setup.roblox.com/versionQTStudio'
const API_URL = 'https://s3.amazonaws.com/setup.roblox.com/$version-API-Dump.json'
const ROOT_NAME = config.rootName
const SEPARATOR = '|'

let watchers = []
let lastUnix = Date.now()

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
            parent = dir[i] + SEPARATOR + parent
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
    let files = uri.files

    for (uri of files) {
        uri = path.parse(uri.fsPath)
        let parent = getParent(uri.dir)

        if (verify(parent) != true) {
            return
        }

        if (uri.name.startsWith('.source') && parent.includes(SEPARATOR) == false) {
            return
        }
    
        events.create(uri.ext, uri.name, parent)
    }
}

function onSave(uri) {
    let source = uri.getText()
    uri = path.parse(uri.fileName)
    let parent = getParent(uri.dir)

    if (verify(parent) != true) {
        return
    }

    if (uri.name.startsWith('.source')) {
        if (parent.includes(SEPARATOR)) {
            events.update(parent, source)
        }
    }
    else {
        events.update(parent + SEPARATOR + uri.name, source)
    }
}

function onDelete(uri) {
    let files = uri.files

    for (uri of files) {
        uri = path.parse(uri.fsPath)
        let parent = getParent(uri.dir)
    
        if (verify(parent) != true) {
            return
        }
    
        if (uri.name.startsWith('.source')) {
            if (parent.includes(SEPARATOR)) {
                events.remove(parent)
                fs.rmdirSync(uri.dir)
            }
        }
        else {
            events.remove(parent + SEPARATOR + uri.name)
        }
    }
}

function onRename(uri) {
    let files = uri.files

    for (uri of files) {
        let newUri = path.parse(uri.newUri.fsPath)
        let oldUri = path.parse(uri.oldUri.fsPath)
        let newParent = getParent(newUri.dir)
        let oldParent = getParent(oldUri.dir)
        
        if (verify(newParent) != true || verify(oldParent) != true) {
            return
        }

        if ((newUri.name.startsWith('.source') || oldUri.name.startsWith('.source')) && (newParent.includes(SEPARATOR) == false || oldParent.includes(SEPARATOR) == false)) {
            return
        }
    
        if (newUri.name != oldUri.name) {
            if (newUri.ext == '.lua' || newUri.ext == '.luau') {
                let newSplitted = newUri.name.split('.')
                let oldSplitted = oldUri.name.split('.')
                
                if (newSplitted.length != oldSplitted.length) {
                    events.changeType(oldParent + SEPARATOR + oldUri.name, newSplitted[newSplitted.length - 1], newUri.name)
                }
                else {
                    let newName = newSplitted[0]
                    let newType = newSplitted[newSplitted.length - 1]
                    let oldName = oldSplitted[0]
                    let oldType = oldSplitted[newSplitted.length - 1]
                    
                    if (newName != oldName && newType == oldType) {
                        events.rename(oldParent + SEPARATOR + oldUri.name, newUri.name)
                    }
                    else if (newType != oldType && newName == oldName) {
                        events.changeType(newParent + SEPARATOR + newUri.name, newType)
                    }
                    else {
                        events.changeType(oldParent + SEPARATOR + oldUri.name, newSplitted[newSplitted.length - 1], newUri.name)
                    }
                }
            }
            else {
                events.rename(oldParent + SEPARATOR + oldUri.name, newUri.name)
            }
        }
        else if (newUri.ext != oldUri.ext) {
            events.changeType(newParent + SEPARATOR + newUri.name, newUri.ext)
        }
        else {
            if (newUri.name.startsWith('.source')) {
                setTimeout(() => {
                    fs.renameSync(uri.newUri.fsPath, uri.oldUri.fsPath)
                }, 100)
            }
            else {
                events.changeParent(oldParent + SEPARATOR + oldUri.name, newParent)
            }
        }
    }
}

function run() {
    let gameDir = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, config.rootName)
    let dataDir = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, '.argon.json')

    if (fs.existsSync(gameDir) == false) {
        fs.mkdirSync(gameDir)
    }

    if (fs.existsSync(dataDir)) {
        let json = JSON.parse(fs.readFileSync(dataDir).toString())

        if (json.classes) {
            events.setTypes(json.classes)
        }
    }
    else {
        events.setTypes(types)
    }

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
    function getVersion(callback) {
        https.get(VERSION_URL, (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                callback(body)
            })
        
        }).on('error', () => {
            messageHandler.showMessage('serverConnection', 2)
        })
    }

    function getClasses(version, dir) {
        https.get(API_URL.replace('$version', version), (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                let classes = JSON.parse(body).Classes
                let newTypes = []
                let newJson = {}
            
                for (let i = 0; i < classes.length; i++) {
                    if (classes[i].Tags == undefined) {
                        newTypes.push(classes[i].Name)
                    }
                    else if (classes[i].Tags.includes('NotCreatable') == false) {
                        newTypes.push(classes[i].Name)
                    }
                }
            
                newJson.version = version
                newJson.classes = newTypes
            
                events.setTypes(newTypes)
                fs.writeFileSync(dir, JSON.stringify(newJson, null, '\t'))
                messageHandler.showMessage('databaseUpdated')
            })
        
        }).on('error', () => {
            messageHandler.showMessage('serverConnection', 2)
        })
    }

    let dir = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, '.argon.json')
    messageHandler.showMessage('updatingDatabase', 1)

    if (fs.existsSync(dir)) {
        let json = JSON.parse(fs.readFileSync(dir).toString())

        getVersion(function(version) {
            if (json.version == version) {
                messageHandler.showMessage('databaseUpToDate')
            }
            else {
                getClasses(version, dir)
            }
        })
    }
    else {
        getVersion(function(version) {
            getClasses(version, dir)
        })
    }
}

function createInstances(dir, instances) {
    for (let [key, value] of instances) {
        let folder = path.join(dir, key)
        value = new Map(Object.entries(value))

        if (fs.existsSync(folder) == false) {
            if (key.endsWith('.Script')) {
                folder = folder.slice(0, -7)

                if (value.size == 0) {
                    fs.writeFileSync(folder + '.server' + config.extension, '')
                }
                else {
                    fs.mkdirSync(folder)
                    fs.writeFileSync(path.join(folder, '.source.server' + config.extension), '')
                }
            }
            else if (key.endsWith('.LocalScript')) {
                folder = folder.slice(0, -12)

                if (value.size == 0) {
                    fs.writeFileSync(folder + '.client' + config.extension, '')
                }
                else {
                    fs.mkdirSync(folder)
                    fs.writeFileSync(path.join(folder, '.source.client' + config.extension), '')
                }
            }
            else if (key.endsWith('.ModuleScript')) {
                folder = folder.slice(0, -13)

                if (value.size == 0) {
                    fs.writeFileSync(folder + config.extension, '')
                }
                else {
                    fs.mkdirSync(folder)
                    fs.writeFileSync(path.join(folder, '.source' + config.extension), '')
                }
            }
            else {
                fs.mkdirSync(folder)
            }
        }

        if (value.size > 0) {
            setTimeout(() => {
                createInstances(folder, value)
            }, 100)
        }
    }

    lastUnix = Date.now()
}

async function portInstances(instances) {
    let dir = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, config.rootName)
    instances = new Map(Object.entries(JSON.parse(instances)))
    createInstances(dir, instances)
}

function portScripts(scripts) {
    scripts = JSON.parse(scripts)

    for (let script of scripts) {
        let dir = path.join(path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, config.rootName), script.Instance)

        if (fs.existsSync(dir + config.extension)) {
            fs.writeFileSync(dir + config.extension, script.Source)
            console.log(3);
        }
        else {
            switch (script.Type) {
                case 'Script':
                    var localDir = path.join(dir, '.source.server') + config.extension
                    if (fs.existsSync(localDir)) {
                        fs.writeFileSync(localDir, script.Source)
                        console.log(3);
                    }
                    else {
                        console.log(script.Instance);
                    }
                    break
                case 'LocalScript':
                    var localDir = path.join(dir, '.source.client') + config.extension
                    if (fs.existsSync(localDir)) {
                        fs.writeFileSync(localDir, script.Source)
                        console.log(3);
                    }
                    else {
                        console.log(script.Instance);
                    }
                    break
                case 'ModuleScript':
                    var localDir = path.join(dir, '.source') + config.extension
                    if (fs.existsSync(localDir)) {
                        fs.writeFileSync(localDir, script.Source)
                        console.log(3);
                    }
                    else {
                        console.log(script.Instance);
                    }
                    break
            }
        }
    }

    lastUnix = Date.now()
}

function getUnix() {
    return lastUnix
}

module.exports = {
    run,
    stop,
    updateClasses,
    portInstances,
    portScripts,
    getUnix
}