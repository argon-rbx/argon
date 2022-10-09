const types = require('../config/types')

let queue = []

function onCreate(name, ext) {
    if (ext == '.lua') {
        let type = name.split('.')
        type = type[type.length - 1]

        switch (type) {
            case 'client':
                name = name.replace('.client', '')
                queue.push({type: type, name: name})
                break
            case 'server':
                name = name.replace('.server', '')
                queue.push({type: type, name: name})
                break
            default:
                queue.push({type: 'module', name: name})
                break
        }

    }

    if (types.includes(ext)) {
        queue.push({type: ext, name: name})
    }
}

function onChange() {
    console.log('file changed')
}

function onDelete() {
    console.log('file deleted')
}

module.exports = {
    onCreate,
    onChange,
    onDelete
}