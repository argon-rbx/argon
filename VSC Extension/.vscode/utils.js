const vscode = require('vscode')
const https = require('https')
const path = require('path')
const fs = require('fs')

const VERSION_URL = 'https://s3.amazonaws.com/setup.roblox.com/versionQTStudio'
const API_URL = 'https://s3.amazonaws.com/setup.roblox.com/$version-API-Dump.json'
const DIR = path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, 'temp.json')

function getVersion(callback) {
    https.get(VERSION_URL, (response) => {
        let body = ''
    
        response.on('data', (data) => {
            body += data
        })

        response.on('end', () => {
            callback(body)
        })
    
    }).on('error', () => {
        console.log('ERROR: getVersion');
    })
}

function getClasses() {
    getVersion((version) => {
        https.get(API_URL.replace('$version', version), (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                let classes = JSON.parse(body).Classes
                let newTypes = []
            
                for (let type of classes) {
                    if (!type.Tags) {
                        newTypes.push(type.Name)
                    }
                    else if (!type.Tags.includes('NotCreatable')) {
                        newTypes.push(type.Name)
                    }
                }

                newTypes.push('StarterCharacterScripts')
                newTypes.push('StarterPlayerScripts')

                console.log(newTypes);
            })
        
        }).on('error', () => {
            console.log('ERROR: getClasses');
        })
    })
}

function getProperties() {
    getVersion((version) => {
        https.get(API_URL.replace('$version', version), (response) => {
            let body
            body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                body = JSON.parse(body)

                let classes = body.Classes
                let enums = body.Enums

                let properties = new Map()
                let schema = {}
            
                for (let type of classes) {
                    if (type.length != 0 && (!type.Tags || !type.Tags.includes('NotCreatable'))) {
                        for (let member of type.Members) {
                            if (member.MemberType == 'Property' && !properties.get(member.Name)) {
                                if (!member.Tags || (!member.Tags.includes('ReadOnly') && !member.Tags.includes('NotScriptable') && !member.Tags.includes('Deprecated') && !member.Tags.includes('Hidden'))) {
                                    properties.set(member.Name, {
                                        Name: member.ValueType.Name,
                                        Category: member.ValueType.Category
                                    })
                                }
                            }
                        }
                    }
                }

                function getEnumValues(name, items) {
                    let values = []

                    for (let item of items) {
                        values.push('Enum.' + name + '.' + item.Name)
                    }

                    return values
                }

                function addSchema(name, type, enumName) {
                    if (!enumName) {
                        schema[name] = {
                            type: type
                        }
                    }
                    else {
                        for (let robloxEnum of enums) {
                            if (robloxEnum.Name == enumName) {
                                schema[name] = {
                                    type: type,
                                    enum: getEnumValues(name, robloxEnum.Items)
                                }
                                break
                            }
                        }
                    }
                }

                let temp = []

                for (let [key, value] of properties) {
                    switch(value.Category) {
                        case 'Primitive':
                            switch (value.Name) {
                                case 'bool':
                                    addSchema(key, 'boolean')
                                    break
                                case 'string':
                                    addSchema(key, 'string')
                                    break
                                case 'int':
                                case 'int64':
                                    addSchema(key, 'integer')
                                    break
                                case 'float':
                                case 'double':
                                    addSchema(key, 'number')
                                    break
                            }
                            break
                        case 'Enum':
                            addSchema(key, 'string', value.Name)
                            break
                        case 'Class':
                            addSchema(key, 'string')
                            break
                        default:
                            if (!temp.includes(value.Name)) {
                                temp.push(value.Name)
                            }
                            break
                    }
                }

                console.log(temp);

                //console.log(schema);
                //fs.writeFileSync(DIR, JSON.stringify(schema, null, '\t'))
            })
        
        }).on('error', () => {
            console.log('ERROR: getProperties');
        })
    })
}

module.exports = {
    getProperties,
    getClasses
}