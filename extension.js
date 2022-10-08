const vscode = require('vscode');
const server = require('./js/server.js')

/**
 * @param {vscode.ExtensionContext} context
 */
async function activate(context) {
	let runCommand = vscode.commands.registerCommand('argon.run', function () {
		server.run()
		vscode.window.showInformationMessage('Server running!');
	});
	
	let stopCommand = vscode.commands.registerCommand('argon.stop', function () {
		server.stop()
		vscode.window.showInformationMessage('Server stopped!');
	});

	//temp
	server.run()

	context.subscriptions.push(runCommand);
	context.subscriptions.push(stopCommand);
}

function deactivate() {}

module.exports = {
	activate,
	deactivate
}