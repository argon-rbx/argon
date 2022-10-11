const vscode = require('vscode')
const main = require('./js/main.js')
const config = require('./config/settings.js')

/**
 * @param {vscode.ExtensionContext} context
 */
async function activate(context) {
	let runCommand = vscode.commands.registerCommand('argon.run', main.run)
	let stopCommand = vscode.commands.registerCommand('argon.stop', main.stop)
	let updateCommand = vscode.commands.registerCommand('argon.update', main.update)

	context.subscriptions.push(runCommand, stopCommand, updateCommand)

	if (config.autoRun) {
		main.run()
	}
}

function deactivate() {}

module.exports = {
	activate,
	deactivate
}