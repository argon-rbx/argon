const https = require('https')
const vscode = require('vscode')
const files = require('./files')
const server = require('./server')
const messageHandler = require('./messageHandler')
const { json } = require('stream/consumers')

const VERSION_URL = 'https://s3.amazonaws.com/setup.roblox.com/versionQTStudio'
const API_URL = 'https://s3.amazonaws.com/setup.roblox.com/$url-API-Dump.json'

function run() {
    if (vscode.workspace.name !== undefined) {
        files.run()
        server.run()
        messageHandler.showMessage('main1')
    }
    else {
        messageHandler.showMessage('main2', 1)
    }
}

function stop()
{
    files.stop()
    server.stop()
    messageHandler.showMessage('main3')
}

function update()
{
    function getData(url) {
        https.get(url, (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                try {
                    if (url == VERSION_URL) {
                        getData(API_URL.replace('$url', body))
                    }
                    else {
                        let classes = JSON.parse(body).Classes
                        let types = []

                        for (let i = 0; i < classes.length; i++) {
                            if (classes[i].Tags == undefined) {
                                types.push(classes[i].Name)
                            }
                            else if (classes[i].Tags.includes('NotCreatable') == false) {
                                types.push(classes[i].Name)
                            }
                        }

                        console.log(types);
                    }
                } catch (error) {
                    messageHandler.showMessage('main4', 2)
                }
            })
        
        }).on('error', () => {
            messageHandler.showMessage('main5', 2)
        })
    }

    getData(VERSION_URL)
}

module.exports = {
    run,
    stop,
    update
}