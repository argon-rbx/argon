const messages = require('../config/messages')
const window = require('vscode').window

function showMessage(message, mode) {
    switch (mode){
        case 1:
            window.showWarningMessage(messages[message])
            break
        case 2:
            window.showErrorMessage(messages[message])
            break
        default:
            window.showInformationMessage(messages[message])
            break
    }
}

module.exports = {
    showMessage
}