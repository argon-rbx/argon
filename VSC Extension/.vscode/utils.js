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

function generateClasses() {
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

                fs.writeFileSync(DIR, JSON.stringify(newTypes, null, '\t'))
            })
        
        }).on('error', () => {
            console.log('ERROR: getClasses');
        })
    })
}

function generateSchema() {
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
                let types = []

                schema['$schema'] = 'http://json-schema.org/draft-07/schema#'
                schema['type'] = 'object'
                schema['properties'] = {}
            
                for (let type of classes) {
                    if (type.length != 0 && (!type.Tags || !type.Tags.includes('NotCreatable'))) {
                        do {
                            if (!types.includes(type.Name)) {
                                for (let member of type.Members) {
                                    if (member.MemberType == 'Property') {
                                        if (!member.Tags || (!member.Tags.includes('ReadOnly') && !member.Tags.includes('NotScriptable') && !member.Tags.includes('Deprecated') && !member.Tags.includes('Hidden'))) {
                                            if (!properties.get(member.Name)) {
                                                properties.set(member.Name, {
                                                    Name: member.ValueType.Name,
                                                    Category: member.ValueType.Category
                                                })
                                            }
                                            else {
                                                if (Array.isArray(properties.get(member.Name))) {
                                                    if (!properties.get(member.Name).Name.includes(member.ValueType.Name)) {
                                                        properties.set(member.Name, {
                                                            Name: properties.get(member.Name).Name.concat(member.ValueType.Name),
                                                            Category: 'Misc'
                                                        })
                                                    }
                                                }
                                                else {
                                                    if (!properties.get(member.Name).Name != member.ValueType.Name) {
                                                        properties.set(member.Name, {
                                                            Name : [properties.get(member.Name).Name, member.ValueType.Name],
                                                            Category: 'Misc'
                                                        })
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            for (let superclass of classes) {
                                if (superclass.Name == type.Superclass) {
                                    types.push(type.Name)
                                    type = superclass
                                    break
                                }
                            }
                        } while (type.Superclass != '<<<ROOT>>>');
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
                        schema.properties[name] = {
                            type: type
                        }
                    }
                    else {
                        for (let robloxEnum of enums) {
                            if (robloxEnum.Name == enumName) {
                                schema.properties[name] = {
                                    type: type,
                                    enum: getEnumValues(enumName, robloxEnum.Items)
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
                        case 'DataType':
                            if (!temp.includes(value.Name)) {
                                temp.push(value.Name)
                            }
                            addSchema(key, 'array')
                            break
                        case 'Enum':
                            addSchema(key, 'string', value.Name)
                            break
                        case 'Class':
                            addSchema(key, 'string')
                            break
                        case 'Misc':
                            console.log(value.Name);
                    }
                }

                console.log(temp);

                fs.writeFileSync(DIR, JSON.stringify(schema, null, '\t'))
            })
        
        }).on('error', () => {
            console.log('ERROR: generateSchema');
        })
    })
}

module.exports = {
    generateClasses,
    generateSchema
}