const types = require('../config/types')

let queue = []

function onCreate(ext, name, parent) {
    if (ext == '.lua') {
        let type = name.split('.')
        type = type[type.length - 1]

        switch (type) {
            case 'client':
                name = name.replace('.client', '')
                queue.push({type: type, name: name, parent: parent})
                break
            case 'server':
                name = name.replace('.server', '')
                queue.push({type: type, name: name, parent: parent})
                break
            default:
                queue.push({type: 'module', name: name, parent: parent})
                break
        }

    }

    if (types.includes(ext)) {
        ext = ext.substring(1)
        queue.push({type: ext, name: name, parent: parent})
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