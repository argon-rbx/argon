const fs = require('fs')
const path = require('path')
const https = require('https')
const vscode = require('vscode')
const events = require('./events')
const config = require('../config/settings.js')
const types = require('../config/types.js')
const messageHandler = require('./messageHandler')

const VERSION_URL = 'https://s3.amazonaws.com/setup.roblox.com/versionQTStudio'
const API_URL = 'https://s3.amazonaws.com/setup.roblox.com/$version-API-Dump.json'
const ARGON_JSON = '.argon.json'
const SEPARATOR = '|'

let watchers = []
let filesToSync = []
let lastUnix = Date.now()

function verify(parent) {
    if (parent == null || parent == '' || parent == 'StarterPlayer') {
        return true
    }
}

function getParent(root) {
    if (root.includes(config.rootName) == false) {
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

        if (dir[i] == config.rootName) {
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

    parent = parent.slice(config.rootName.length + 1)
    return parent
}

function onCreate(uri) {
    let files = uri.files

    for (uri of files) {
        uri = path.parse(uri.fsPath)
        let parent = getParent(uri.dir)

        if (verify(parent)) {
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

    if (verify(parent)) {
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
    
        if (verify(parent)) {
            return
        }
    
        if (uri.name.startsWith('.source')) {
            if (parent.includes(SEPARATOR)) {
                events.remove(parent)
                fs.rmSync(uri.dir, {recursive: true})
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
        
        if (verify(newParent) || verify(oldParent)) {
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
    let dataDir = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, ARGON_JSON)

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
    for (let watcher of watchers) {
        watcher.dispose()
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
            
                for (let type of classes) {
                    if (type.Tags == undefined) {
                        newTypes.push(type.Name)
                    }
                    else if (type.Tags.includes('NotCreatable') == false) {
                        newTypes.push(type.Name)
                    }
                }

                newTypes.push('StarterCharacterScripts')
                newTypes.push('StarterPlayerScripts')

                if (fs.existsSync(dir)) {
                    let json = JSON.parse(fs.readFileSync(dir).toString())

                    if (json.directory) {
                        newJson.directory = json.directory
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

    let dir = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, ARGON_JSON)
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

        if (key == 'forceSubScript') {
            continue
        }

        if (key.endsWith('.Script')) {
            folder = folder.slice(0, -7)

            if (value.size == 0) {
                fs.writeFileSync(folder + '.server' + config.extension, '')
            }
            else {
                if (fs.existsSync(folder) == false) {
                    fs.mkdirSync(folder)
                }

                fs.writeFileSync(path.join(folder, '.source.server' + config.extension), '')
            }
        }
        else if (key.endsWith('.LocalScript')) {
            folder = folder.slice(0, -12)

            if (value.size == 0) {
                fs.writeFileSync(folder + '.client' + config.extension, '')
            }
            else {
                if (fs.existsSync(folder) == false) {
                    fs.mkdirSync(folder)
                }

                fs.writeFileSync(path.join(folder, '.source.client' + config.extension), '')
            }
        }
        else if (key.endsWith('.ModuleScript')) {
            folder = folder.slice(0, -13)

            if (value.size == 0) {
                fs.writeFileSync(folder + config.extension, '')
            }
            else {
                if (fs.existsSync(folder) == false) {
                    fs.mkdirSync(folder)
                }

                fs.writeFileSync(path.join(folder, '.source' + config.extension), '')
            }
        }
        else {
            if (fs.existsSync(folder) == false) {
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

function getRootDir() {
    let rootDir = vscode.workspace.workspaceFolders[0].uri.fsPath
    let jsonDir = path.join(rootDir, ARGON_JSON)

    if (fs.existsSync(jsonDir)) {
        let json = JSON.parse(fs.readFileSync(jsonDir).toString())

        if (json.directory) {
            rootDir = path.join(rootDir, json.directory)
        }
    }

    rootDir = path.join(rootDir, config.rootName)

    if (fs.existsSync(rootDir) == false && config.autoCreateFolder) {
        fs.mkdirSync(rootDir)
    }

    return rootDir
}

function portInstances(data) {
    data = JSON.parse(data)

    let dir = getRootDir()
    let instances = data.instances
    let mode = data.mode

    if (mode) {
        instances = new Map(Object.entries(instances))
        for (let [key, value] of instances) {
            let folder = path.join(dir, key)

            if (fs.existsSync(folder) == false) {
                fs.mkdirSync(folder)
            }

            for (let instance of value) {
                instance = new Map(Object.entries(instance))
                createInstances(folder, instance)
            }
        }
    }
    else {
        instances = new Map(Object.entries(instances))
        createInstances(dir, instances)
    }
}

function portScripts(scripts) {
    scripts = JSON.parse(scripts)

    for (let script of scripts) {
        let dir = path.join(getRootDir(), script.Instance)

        if (fs.existsSync(dir + config.extension)) {
            fs.writeFileSync(dir + config.extension, script.Source)
        }
        else {
            switch (script.Type) {
                case 'Script':
                    var localDir = path.join(dir, '.source.server') + config.extension
                    if (fs.existsSync(localDir)) {
                        fs.writeFileSync(localDir, script.Source)
                    }
                    break
                case 'LocalScript':
                    var localDir = path.join(dir, '.source.client') + config.extension
                    if (fs.existsSync(localDir)) {
                        fs.writeFileSync(localDir, script.Source)
                    }
                    break
                case 'ModuleScript':
                    var localDir = path.join(dir, '.source') + config.extension
                    if (fs.existsSync(localDir)) {
                        fs.writeFileSync(localDir, script.Source)
                    }
                    break
            }
        }
    }

    lastUnix = Date.now()
}

function portCreate(uri) {
    uri = path.parse(uri)
    let parent = getParent(uri.dir)

    if (verify(parent)) {
        return
    }

    if (uri.name.startsWith('.source') && parent.includes(SEPARATOR) == false) {
        return
    }

    events.create(uri.ext, uri.name, parent)
}

function portSave(uri) {
    let parsedUri = path.parse(uri)
    let parent = getParent(parsedUri.dir)

    if (verify(parent)) {
        return
    }

    let source = fs.readFileSync(uri, 'utf-8')

    if (parsedUri.name.startsWith('.source')) {
        if (parent.includes(SEPARATOR)) {
            filesToSync.push(events.portSource(parent, source))
        }
    }
    else {
        filesToSync.push(events.portSource(parent + SEPARATOR + parsedUri.name, source))
    }
}

function getSubDirs(uri) {
    fs.readdirSync(uri, {withFileTypes: true}).forEach(file => {
        let subUri = path.join(uri, file.name)

        portCreate(subUri)

        if (file.isDirectory()) {
            getSubDirs(subUri)
        }
        else {
            portSave(subUri)
        }
    })
}

function getChunk(data, index) {
    let chunk, lastChunk
    chunk = lastChunk = []

    for (let i = index; i < data.length; i++) {
        index = i

        chunk.push(data[i])

        if (JSON.stringify(chunk).length / 1000 < 1020) {
            lastChunk.push(data[i])
        }
        else {
            return [lastChunk, index]
        }
    }

    return [lastChunk, index]
}

function portProject() {
    let dir = getRootDir()
    let chunks = []
    let index = 0

    filesToSync.length = 0

    fs.readdirSync(dir).forEach(file => {
        let uri = path.join(dir, file)

        portCreate(uri)
        getSubDirs(uri)
    })

    while (index != filesToSync.length - 1) {
        let chunk
        [chunk, index] = getChunk(filesToSync, index)
        chunks.push(chunk)
    }

    return chunks
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
    portProject,
    getUnix
}