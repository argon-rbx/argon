const types = require('../config/types')

let queue = []

function onCreate(ext, name, parent) {
    if (ext == '.lua') {
        let type = name.split('.')
        type = type[type.length - 1]

        switch (type) {
            case 'client':
                name = name.replace('.client', '')
                queue.push({Action: 'create', Type: type, Name: name, Parent: parent})
                break
            case 'server':
                name = name.replace('.server', '')
                queue.push({Action: 'create', Type: type, Name: name, Parent: parent})
                break
            default:
                queue.push({Action: 'create', Type: 'module', Name: name, Parent: parent})
                break
        }

    }

    if (types.includes(ext)) {
        ext = ext.substring(1)
        queue.push({Action: 'create', Type: ext, Name: name, Parent: parent})
    }
    else if (ext == '') {
        queue.push({Action: 'create', Type: 'folder', Name: name, Parent: parent})
    }
}

function onSave(object, type, source) {
    queue.push({Action: 'update', Type: type, Object: object, Source: source})
}

function onDelete(object) {
    console.log(object)
    queue.push({Action: 'delete', Object: object})
}

module.exports = {
    queue,
    onCreate,
    onSave,
    onDelete
}