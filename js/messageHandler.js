const vscode = require('vscode')
const messages = require('../config/messages')

function showMessage(message, mode) {
    switch (mode){
        case 1:
            vscode.window.showWarningMessage(messages[message])
            break
        case 2:
            vscode.window.showErrorMessage(messages[message])
            break
        default:
            vscode.window.showInformationMessage(messages[message])
            break
    }
}

module.exports = {
    showMessage
}