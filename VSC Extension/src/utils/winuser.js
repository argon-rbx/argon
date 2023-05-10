// @ts-nocheck
const config = require('../config/settings')
const messageHandler = require('../messageHandler')

function getVersion() {
    switch (config.nodeModules) {
        case '106':
            return require('./winuser-106')
        case '110':
            return require('./winuser-110')
        default:
            messageHandler.show('unsupportedVersion', 2)
            return null
        }
}

if (config.os == 'win32') {
    module.exports = getVersion()
}
else {
    module.exports = null
}