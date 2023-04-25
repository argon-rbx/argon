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
        console.log('ERROR: getVersion')
    })
}

function generateJsonSchema(dir) {
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
                schema['properties'] = {Class: {type: 'string'}}
            
                for (let type of classes) {
                    if (type.length != 0 && (!type.Tags || !type.Tags.includes('NotCreatable'))) {
                        do {
                            if (!types.includes(type.Name)) {
                                for (let member of type.Members) {
                                    if (member.MemberType == 'Property') {
                                        if (!member.Tags || (!member.Tags.includes('ReadOnly') && !member.Tags.includes('NotScriptable') && !member.Tags.includes('Deprecated') && !member.Tags.includes('Hidden'))) {
                                            if (!properties.has(member.Name)) {
                                                properties.set(member.Name, {
                                                    Name: member.ValueType.Name,
                                                    Category: member.ValueType.Category
                                                })
                                            }
                                            else {
                                                if (properties.get(member.Name).Types) {
                                                    if (!properties.get(member.Name).Types.has(member.ValueType.Name)) {
                                                        let types = properties.get(member.Name).Types
                                                        types.set(member.ValueType.Name, member.ValueType.Category)
                                                        properties.set(member.Name, {
                                                            Types: types,
                                                            Category: 'Misc'
                                                        })
                                                    }
                                                }
                                                else {
                                                    if (properties.get(member.Name).Name != member.ValueType.Name) {
                                                        let types = new Map()
                                                        types.set(properties.get(member.Name).Name, properties.get(member.Name).Category)
                                                        types.set(member.ValueType.Name, member.ValueType.Category)
                                                        properties.set(member.Name, {
                                                            Types: types,
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
                        } while (type.Superclass != '<<<ROOT>>>')
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
                            if (value.Name != 'Content' && value.Name != 'BrickColor') {
                                addSchema(key, 'array')
                            }
                            else {
                                addSchema(key, 'string')
                            }
                            break
                        case 'Enum':
                            addSchema(key, 'string', value.Name)
                            break
                        case 'Class':
                            addSchema(key, 'string')
                            break
                        case 'Misc':
                            let types = []

                            for (let [name, category] of value.Types) {
                                switch (category) {
                                    case 'Primitive':
                                        switch (name) {
                                            case 'bool':
                                                if (!types.includes('boolean')) {
                                                    types.push('boolean')
                                                }
                                                break
                                            case 'string':
                                                if (!types.includes('string')) {
                                                    types.push('string')
                                                }
                                                break
                                            case 'int':
                                            case 'int64':
                                                if (!types.includes('integer')) {
                                                    types.push('integer')
                                                }
                                                break
                                            case 'float':
                                            case 'double':
                                                if (!types.includes('number')) {
                                                    types.push('number')
                                                }
                                                break
                                        }
                                        break
                                    case 'DataType':
                                        if (value.Name != 'Content' && value.Name != 'BrickColor') {
                                            if (!types.includes('array')) {
                                                types.push('array')
                                            }
                                        }
                                        else {
                                            if (!types.includes('string')) {
                                                types.push('string')
                                            }
                                        }
                                        break
                                    default:
                                        if (!types.includes('string')) {
                                            types.push('string')
                                        }
                                        break
                                }
                            }

                            if (types.length != 1) {
                                addSchema(key, types)
                            }
                            else {
                                addSchema(key, types[0])
                            }
                            break
                    }
                }

                fs.writeFileSync(dir || DIR, JSON.stringify(schema, null, '\t'))
            })
        
        }).on('error', () => {
            console.log('ERROR: generateJsonSchema')
        })
    })
}

function generateApiDump(dir) {
    getVersion((version) => {
        if (dir) {
            dir = path.join(dir, 'src' + path.sep + 'config' + path.sep + 'apiDump.js')

            if (fs.existsSync(dir)) {
                let apiDump = fs.readFileSync(dir).toString()
                let currentVersion = apiDump.substring(apiDump.indexOf('//') + 2, apiDump.lastIndexOf('//'))

                if (version == currentVersion) {
                    return
                }
            }
        }

        https.get(API_URL.replace('$version', version), (response) => {
            let body = ''
        
            response.on('data', (data) => {
                body += data
            })
    
            response.on('end', () => {
                let classes = JSON.parse(body).Classes
                let apiDump = {}

                for (let type of classes) {
                    let properties = []
                    let newType = type

                    do {
                        for (let member of newType.Members) {
                            if (member.MemberType == 'Property') {
                                if (!member.Tags || (!member.Tags.includes('ReadOnly') && !member.Tags.includes('NotScriptable') && !member.Tags.includes('Deprecated') && !member.Tags.includes('Hidden'))) {
                                    if (!properties.includes(member.Name)) {
                                        properties.push(member.Name)
                                    }
                                }
                            }
                        }

                        for (let superclass of classes) {
                            if (superclass.Name == newType.Superclass) {
                                newType = superclass
                                break
                            }
                        }
                    } while (newType.Superclass != '<<<ROOT>>>')

                    if (properties.length != 0) {
                        apiDump[type.Name] = properties
                    }
                }

                if (dir) {
                    let content = '//' + version + '//\n' + 'module.exports = '
                    fs.writeFileSync(dir, content + JSON.stringify(apiDump))
                }
                else {
                    fs.writeFileSync(DIR, JSON.stringify(apiDump))
                }
            })
        
        }).on('error', () => {
            console.log('ERROR: generateApiDump')
        })
    })
}

module.exports = {
    generateJsonSchema,
    generateApiDump
}