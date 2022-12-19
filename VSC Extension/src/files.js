const fs = require('fs')
const path = require('path')
const vscode = require('vscode')
const events = require('./events')
const types = require('./config/classes')
const config = require('./config/settings')
const project = require('./config/project')
const messageHandler = require('./messageHandler')

let watchers = []
let filesToSync = []
let lastUnix = Date.now()

function verify(parent) {
    if (!parent || parent == '' || parent == 'StarterPlayer') {
        return true
    }
}

function getParent(root) {
    if (!root.includes(config.rootFolder)) {
        return null
    }

    let dir = root.split('\\')
    let similarName = 0
    let parent = ''

    for (let i = dir.length - 1; i >= 0; i--) {
        if (dir[i].includes('.')) {
            let type = dir[i].split('.')
            type = type[type.length - 1]

            if (types.includes(type)) {
                dir[i] = dir[i].replace('.' + type, '')
            }
        }

        if (i != dir.length - 1) {
            parent = dir[i] + config.separator + parent
        }
        else {
            parent = dir[i]
        }

        if (dir[i] == config.rootFolder) {
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

    parent = parent.slice(config.rootFolder.length + 1)
    return parent
}

function onCreate(uri) {
    let files = uri.files

    for (uri of files) {
        uri = path.parse(uri.fsPath)
        let parent = getParent(uri.dir)

        if (verify(parent) || uri.ext == '.json') {
            return
        }

        if (uri.name.startsWith('.source') && !parent.includes(config.separator)) {
            return
        }
    
        events.create(uri.ext, uri.name, parent)
    }
}

function onSave(uri) {
    let source = uri.getText()
    uri = path.parse(uri.fileName)
    let parent = getParent(uri.dir)

    if (uri.ext != '.json') {
        if (verify(parent)) {
            return
        }

        if (uri.name.startsWith('.source')) {
            if (parent.includes(config.separator)) {
                events.update(parent, source)
            }
        }
        else {
            events.update(parent + config.separator + uri.name, source)
        }
    }
    else if (uri.name == config.properties) {
        if (!parent || parent == '') {
            return
        }

        events.setProperties(parent, source)
    }
}

function onDelete(uri) {
    let files = uri.files

    for (uri of files) {
        uri = path.parse(uri.fsPath)
        let parent = getParent(uri.dir)
    
        if (verify(parent) || uri.ext == '.json') {
            return
        }
    
        if (uri.name.startsWith('.source')) {
            if (parent.includes(config.separator)) {
                events.remove(parent)
                fs.rmSync(uri.dir, {recursive: true})
            }
        }
        else {
            if (uri.ext == '.lua' || uri.ext == '.luau' || types.includes(uri.ext.substring(1))) {
                events.remove(parent + config.separator + uri.name)
            }
            else {
                events.remove(parent + config.separator + uri.name + uri.ext)
            }
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
        
        if (verify(newParent) || verify(oldParent) || newUri.ext == '.json' || oldUri.ext == '.json') {
            return
        }

        if ((newUri.name.startsWith('.source') || oldUri.name.startsWith('.source')) && !(newParent.includes(config.separator) || !oldParent.includes(config.separator))) {
            return
        }
    
        if (newUri.name != oldUri.name) {
            if (newUri.ext == '.lua' || newUri.ext == '.luau') {
                let newSplitted = newUri.name.split('.')
                let oldSplitted = oldUri.name.split('.')
                
                if (newSplitted.length != oldSplitted.length) {
                    events.changeType(oldParent + config.separator + oldUri.name, newSplitted[newSplitted.length - 1], newUri.name)
                }
                else {
                    let newName = newSplitted[0]
                    let newType = newSplitted[newSplitted.length - 1]
                    let oldName = oldSplitted[0]
                    let oldType = oldSplitted[newSplitted.length - 1]
                    
                    if (newName != oldName && newType == oldType) {
                        events.rename(oldParent + config.separator + oldUri.name, newUri.name)
                    }
                    else if (newType != oldType && newName == oldName) {
                        events.changeType(newParent + config.separator + newUri.name, newType)
                    }
                    else {
                        events.changeType(oldParent + config.separator + oldUri.name, newSplitted[newSplitted.length - 1], newUri.name)
                    }
                }
            }
            else {
                events.rename(oldParent + config.separator + oldUri.name, newUri.name)
            }
        }
        else if (newUri.ext != oldUri.ext) {
            events.changeType(newParent + config.separator + newUri.name, newUri.ext)
        }
        else {
            if (newUri.name.startsWith('.source')) {
                setTimeout(() => {
                    fs.renameSync(uri.newUri.fsPath, uri.oldUri.fsPath)
                }, 100)
            }
            else {
                events.changeParent(oldParent + config.separator + oldUri.name, newParent)
            }
        }
    }
}

function run() {
    getRootDir()
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
                if (!fs.existsSync(folder)) {
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
                if (!fs.existsSync(folder)) {
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
                if (!fs.existsSync(folder)) {
                    fs.mkdirSync(folder)
                }

                fs.writeFileSync(path.join(folder, '.source' + config.extension), '')
            }
        }
        else {
            if (!fs.existsSync(folder)) {
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
    let jsonDir = path.join(rootDir, config.json)

    if (fs.existsSync(jsonDir)) {
        let json = JSON.parse(fs.readFileSync(jsonDir).toString())

        if (json.directory) {
            rootDir = path.join(rootDir, json.directory)
        }
    }
    else if (config.autoSetup) {
        fs.writeFileSync(jsonDir, JSON.stringify(project, null, '\t'))
    }

    rootDir = path.join(rootDir, config.rootFolder)

    if (!fs.existsSync(rootDir)) {
        if (config.autoSetup) {
            fs.mkdirSync(rootDir)
        }
        else {
            messageHandler.showMessage('noRootFolder', 2)
        }
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

            if (!fs.existsSync(folder)) {
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

function portProperties(properties) {
    properties = new Map(Object.entries(JSON.parse(properties)))
    let rootDir = getRootDir()

    for (let [key, value] of properties) {
        key = path.join(rootDir, key)
        if (fs.existsSync(key)) {
            value = JSON.stringify(value, null, '\t').replace(/,\n\t\t/g, ', ').replace(/\[\n\t\t/g, '[').replace(/\n\t\]/g, ']').replace(/, \t/g, ', ').replace(/\[\t/g, '[').replace(/\n\t\t\]\]/g, ']]').replace(/\n\t\t\]/g, ']')
            fs.writeFileSync(path.join(key, config.properties + '.json'), value)
        }
    }
}

function portCreate(uri) {
    uri = path.parse(uri)
    let parent = getParent(uri.dir)

    if (verify(parent)) {
        return
    }

    if (uri.name.startsWith('.source') && !parent.includes(config.separator)) {
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
        if (parent.includes(config.separator)) {
            filesToSync.push(events.portSource(parent, source))
        }
    }
    else {
        filesToSync.push(events.portSource(parent + config.separator + parsedUri.name, source))
    }
}

function portUpdate(uri) {
    let parsedUri = path.parse(uri)
    let parent = getParent(parsedUri.dir)

    if (!parent || parent == '') {
        return
    }

    let source = fs.readFileSync(uri, 'utf-8')
    events.setProperties(parent, source)
}

function getSubDirs(uri) {
    fs.readdirSync(uri, {withFileTypes: true}).forEach(file => {
        let subUri = path.join(uri, file.name)

        if (file.name != config.properties + '.json') {
            portCreate(subUri)

            if (file.isDirectory()) {
                getSubDirs(subUri)
            }
            else {
                portSave(subUri)
            }
        }
        else {
            portUpdate(subUri)
        }
    })
}

function getChunk(data, index) {
    let lastChunk = []
    let chunk = []

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

    fs.readdirSync(dir, {withFileTypes: true}).forEach(file => {
        let uri = path.join(dir, file.name)

        if (file.isDirectory()) {
            portCreate(uri)
            getSubDirs(uri)
        }
    })

    do {
        let chunk
        [chunk, index] = getChunk(filesToSync, index)
        chunks.push(chunk)
    } while (index != filesToSync.length - 1);

    filesToSync.length = 0
    return chunks
}

function getUnix() {
    return lastUnix
}

module.exports = {
    run,
    stop,
    portInstances,
    portScripts,
    portProperties,
    portProject,
    getRootDir,
    getUnix
}