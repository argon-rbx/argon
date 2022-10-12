let queue = []
let types = []

function onCreate(ext, name, parent) {
    if (ext == '.lua' || ext == '.luau') {
        let type = name.split('.')
        type = type[type.length - 1]

        switch (type) {
            case 'server':
                name = name.replace('.server', '')
                queue.push({Action: 'create', Type: 'Script', Name: name, Parent: parent})
                break
            case 'client':
                name = name.replace('.client', '')
                queue.push({Action: 'create', Type: 'LocalScript', Name: name, Parent: parent})
                break
            default:
                queue.push({Action: 'create', Type: 'ModuleScript', Name: name, Parent: parent})
                break
        }
    }

    ext = ext.substring(1)

    if (types.includes(ext)) {
        queue.push({Action: 'create', Type: ext, Name: name, Parent: parent})
    }
    else if (ext == '') {
        queue.push({Action: 'create', Type: 'Folder', Name: name, Parent: parent})
    }
}

function onSave(object, source) {
    if (object.includes('.server')) {
        object = object.replace('.server', '')
    }
    else if (object.includes('.client')) {
        object = object.replace('.client', '')
    }

    queue.push({Action: 'update', Object: object, Source: source})
}

function onDelete(object) {
    queue.push({Action: 'delete', Object: object})
}

function setTypes(newTypes) {
    types = newTypes
}

function getTypes() {
    return types
}

module.exports = {
    queue,
    onCreate,
    onSave,
    onDelete,
    setTypes,
    getTypes
}