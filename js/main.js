const vscode = require('vscode')
const files = require('./files')
const messageHandler = require('./messageHandler')

function run() {
    if (vscode.workspace.name !== undefined) {
        files.run()

        messageHandler.showMessage('main1')
    }
    else {
        messageHandler.showMessage('main2', 1)
    }
}

function stop()
{
    files.stop()
    messageHandler.showMessage('main3')
}

module.exports = {
    run,
    stop
}