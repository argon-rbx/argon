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

}

function changeParent(object, parent) {
    object = parse(object)
    console.log(object, parent);
    queue.push({Action: 'changeParent', Object: object, Parent: parent})
}

function setTypes(newTypes) {
    types = newTypes
}

function getTypes() {
    return types
}

module.exports = {
    queue,
    create,
    update,
    remove,
    rename,
    changeType,
    changeParent,
    setTypes,
    getTypes
}