const config = require('./config/settings')

let queue = []

function parse(string) {
    if (string.includes('.server')) {
        string = string.replace('.server', '')
    }
    else if (string.includes('.client')) {
        string = string.replace('.client', '')
    }

    return string
}

function create(ext, name, parent) {
    if (ext == '.lua' || ext == '.luau') {
        let type = name.split('.')
        type = type[type.length - 1]

        switch (type) {
            case 'server':
                name = name.replace('.server', '')

                if (name != config.source) {
                    queue.push({Action: 'create', Type: 'Script', Name: name, Parent: parent})
                }
                else {
                    let splitted = parent.split(config.separator)
                    name = parent.slice(-splitted[splitted.length - 1].length)
                    parent = parent.slice(0, -(name.length + 1))
                    queue.push({Action: 'create', Type: 'Script', Name: name, Parent: parent, Delete: true})
                }

                break
            case 'client':
                name = name.replace('.client', '')

                if (name != config.source) {
                    queue.push({Action: 'create', Type: 'LocalScript', Name: name, Parent: parent})
                }
                else {
                    let splitted = parent.split(config.separator)
                    name = parent.slice(-splitted[splitted.length - 1].length)
                    parent = parent.slice(0, -(name.length + 1))
                    queue.push({Action: 'create', Type: 'LocalScript', Name: name, Parent: parent, Delete: true})
                }

                break
            default:
                if (name != config.source) {
                    queue.push({Action: 'create', Type: 'ModuleScript', Name: name, Parent: parent})
                }
                else {
                    let splitted = parent.split(config.separator)
                    name = parent.slice(-splitted[splitted.length - 1].length)
                    parent = parent.slice(0, -(name.length + 1))
                    queue.push({Action: 'create', Type: 'ModuleScript', Name: name, Parent: parent, Delete: true})
                }

                break
        }
    }
    else {
        queue.push({Action: 'create', Type: 'Folder', Name: name + ext, Parent: parent})
    }
}

function update(object, source) {
    object = parse(object)
    queue.push({Action: 'update', Object: object, Source: source})
}

function remove(object) {
    object = parse(object)
    queue.push({Action: 'delete', Object: object})
}

function rename(object, name) {
    object = parse(object)
    name = parse(name)
    queue.push({Action: 'rename', Object: object, Name: name})
}

function changeType(object, type, name) {
    object = parse(object)
    type = type.replace('.', '')

    if (object.endsWith(config.separator + config.source)) {
        object = object.replace(config.separator + config.source, '')
        let splitted = object.split(config.separator)
        name = object.slice(-splitted[splitted.length - 1].length)
    }

    if (name) {
        name = parse(name)
        switch (type) {
            case 'server':
                queue.push({Action: 'changeType', Object: object, Type: 'Script', Name: name})
                break
            case 'client':
                queue.push({Action: 'changeType', Object: object, Type: 'LocalScript', Name: name})
                break
            default:
                queue.push({Action: 'changeType', Object: object, Type: 'ModuleScript', Name: name})
                break
        }
    }
    else {
        switch (type) {
            case 'server':
                queue.push({Action: 'changeType', Object: object, Type: 'Script',})
                break
            case 'client':
                queue.push({Action: 'changeType', Object: object, Type: 'LocalScript',})
                break
        }
    }
}

function changeParent(object, parent) {
    object = parse(object)
    queue.push({Action: 'changeParent', Object: object, Parent: parent})
}

function setProperties(object, properties) {
    object = parse(object)
    queue.push({Action: 'setProperties', Object: object, Properties: properties})
}

function portSource(object, source) {
    object = parse(object)
    return {Action: 'update', Object: object, Source: source}
}

function closeFile() {
    queue.push({Action: 'closeFile'})
}

function executeSnippet(snippet) {
    queue.push({Action: 'executeSnippet', Snippet: snippet})
}

function setTitle(title) {
    queue.push({Action: 'setTitle', Title: title})
}

module.exports = {
    queue,
    create,
    update,
    remove,
    rename,
    changeType,
    changeParent,
    setProperties,
    portSource,
    closeFile,
    executeSnippet,
    setTitle
}