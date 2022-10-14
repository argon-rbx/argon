const vscode = require('vscode')
const messages = require('../config/messages')

function showMessage(message, mode) {
    message = messages[message]

    if (message.toLowerCase().includes('argon') == false) {
        message = 'Argon: ' + message
    }

    switch (mode){
        case 1:
            vscode.window.showWarningMessage(message)
            break
        case 2:
            vscode.window.showErrorMessage(message)
            break
        default:
            vscode.window.showInformationMessage(message)
            break
    }
}

module.exports = {
    showMessage
}