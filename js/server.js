const vscode = require('vscode')
const http = require('http')
const config = require('../config/settings.js')

const PORT = config.port
const HOST = config.host

let localServer = http.createServer(requestListener)

function requestListener(request, response) {
    console.log(request.rawHeaders)

    response.writeHead(200)
    response.end("My first server!")
}

function run() {
    localServer.listen(PORT, HOST);
    vscode.window.showInformationMessage('Server running!')
}

function stop() {
    localServer.close()
    vscode.window.showInformationMessage('Server stopped!')
}

module.exports = {
    run,
    stop
}