const vscode = require('vscode')
const messages = require('../config/messages')
const config = require('../config/settings.js')

function showMessage(message, mode) {
    if (config.hideNotifications == false) {
        message = messages[message]

        if (message.toLowerCase().includes('Argon') == false) {
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
}

module.exports = {
    showMessage
}