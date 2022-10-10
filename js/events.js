const types = require('../config/types')

let queue = []

function push(action, type, name, parent) {
    queue.push({Action: action, Type: type, Name: name, Parent: parent})
}

function onCreate(ext, name, parent) {
    if (ext == '.lua') {
        let type = name.split('.')
        type = type[type.length - 1]

        switch (type) {
            case 'client':
                name = name.replace('.client', '')
                push('create', type, name, parent)
                break
            case 'server':
                name = name.replace('.server', '')
                push('create', type, name, parent)
                break
            default:
                push('create', 'module', name, parent)
                break
        }

    }

    if (types.includes(ext)) {
        ext = ext.substring(1)
        push('create', ext, name, parent)
    }
    else if (ext == '') {
        push('create', 'folder', name, parent)
    }
}

function onChange() {
    console.log('file changed')
}

function onDelete() {
    console.log('file deleted')
}

module.exports = {
    queue,
    onCreate,
    onChange,
    onDelete
}