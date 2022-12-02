const SEPARATOR = '|'

let queue = []
let types = []

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

                if (name != '.source') {
                    queue.push({Action: 'create', Type: 'Script', Name: name, Parent: parent})
                }
                else {
                    let splitted = parent.split(SEPARATOR)
                    name = parent.slice(-splitted[splitted.length - 1].length)
                    parent = parent.slice(0, -(name.length + 1))
                    queue.push({Action: 'create', Type: 'Script', Name: name, Parent: parent, Delete: true})
                }

                break
            case 'client':
                name = name.replace('.client', '')

                if (name != '.source') {
                    queue.push({Action: 'create', Type: 'LocalScript', Name: name, Parent: parent})
                }
                else {
                    let splitted = parent.split(SEPARATOR)
                    name = parent.slice(-splitted[splitted.length - 1].length)
                    parent = parent.slice(0, -(name.length + 1))
                    queue.push({Action: 'create', Type: 'LocalScript', Name: name, Parent: parent, Delete: true})
                }

                break
            default:
                if (name != '.source') {
                    queue.push({Action: 'create', Type: 'ModuleScript', Name: name, Parent: parent})
                }
                else {
                    let splitted = parent.split(SEPARATOR)
                    name = parent.slice(-splitted[splitted.length - 1].length)
                    parent = parent.slice(0, -(name.length + 1))
                    queue.push({Action: 'create', Type: 'ModuleScript', Name: name, Parent: parent, Delete: true})
                }

                break
        }
    }
    else {
        ext = ext.substring(1)

        if (types.includes(ext)) {
            queue.push({Action: 'create', Type: ext, Name: name, Parent: parent})
        }
        else if (ext == '') {
            queue.push({Action: 'create', Type: 'Folder', Name: name, Parent: parent})
        }
        else if (ext != 'lua' && ext != 'luau') {
            queue.push({Action: 'create', Type: 'Folder', Name: name + '.' + ext, Parent: parent})
        }
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

    if (object.endsWith(SEPARATOR + '.source')) {
        object = object.replace(SEPARATOR + '.source', '')
        let splitted = object.split(SEPARATOR)
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
            default:
                if (types.includes(type)) {
                    queue.push({Action: 'changeType', Object: object, Type: type,})
                }
                break
        }
    }
}

function changeParent(object, parent) {
    object = parse(object)
    queue.push({Action: 'changeParent', Object: object, Parent: parent})
}

function setTypes(newTypes) {
    types = newTypes
}

function getTypes() {
    return types
}

function portSource(object, source) {
    object = parse(object)
    return {Action: 'update', Object: object, Source: source}
}

module.exports = {
    queue,
    types,
    create,
    update,
    remove,
    rename,
    changeType,
    changeParent,
    setTypes,
    getTypes,
    portSource
}