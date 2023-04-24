const fs = require('fs')
const path = require('path')
const files = require('./files')
const config = require('./config/settings')

function createUnknownFolders(dir) {
    let splitted = dir.split(config.osSeparator)
    let tempDir = ''

    for (let folder of splitted) {
        if (folder != splitted[splitted.length - 1]) {
            if (config.os == 'win32' && folder == splitted[0]) {
                folder += config.osSeparator
            }

            tempDir = path.join(tempDir, folder)

            if (!fs.existsSync(tempDir)) {
                fs.mkdirSync(tempDir)
            }
        }
    }
}

function sync(queue) {
    queue = JSON.parse(queue)
    let rootDir = files.getRootDir()

    for (let data of queue) {

        switch (data.Action) {
            case 'sync':
                if (data.Type) {
                    let suffix

                    switch (data.Type) {
                        case 'Script':
                            suffix = config.source + '.server'
                            break
                        case 'LocalScript':
                            suffix = config.source + '.client'
                            break
                        case 'ModuleScript':
                            suffix = config.source
                            break
                    }

                    var dir = files.applyCustomPaths(path.join(rootDir, data.Path + config.osSeparator + suffix + config.extension))
                }
                else {
                    var dir = files.applyCustomPaths(path.join(rootDir, data.Path + config.extension))
                }

                if (fs.existsSync(dir)) {
                    fs.writeFileSync(dir, data.Source)
                }

                break
            case 'changePath':
                if (data.Children && data.Children != 0) {
                    let oldDir = files.applyCustomPaths(path.join(rootDir, data.OldPath))
                    let newDir = files.applyCustomPaths(path.join(rootDir, data.NewPath))

                    if (fs.existsSync(oldDir)) {
                        createUnknownFolders(newDir)
                        fs.renameSync(oldDir, newDir)
                    }
                }
                else {
                    let oldDir = files.applyCustomPaths(path.join(rootDir, data.OldPath + config.extension))
                    let newDir = files.applyCustomPaths(path.join(rootDir, data.NewPath + config.extension))

                    createUnknownFolders(newDir)

                    if (fs.existsSync(oldDir)) {
                        fs.renameSync(oldDir, newDir)
                    }
                    // Create new file
                    else if (data.Source) {
                        fs.writeFileSync(newDir, data.Source)
                    }
                }
                break
            case 'remove':
                if (!data.Type) {
                    var dir = files.applyCustomPaths(path.join(rootDir, data.Path))

                    if (fs.existsSync(dir)) {
                        fs.rmSync(dir, {recursive: true})
                    }

                    return
                }

                var dir = files.applyCustomPaths(path.join(rootDir, data.Path + config.extension))
                let splitted = dir.split(config.osSeparator)

                if (fs.existsSync(dir)) {
                    fs.rmSync(dir)
                }

                let parentName = splitted[splitted.length - 2]
                let scriptDir = dir.replace(splitted[splitted.length - 1], '')
                let parentDir = scriptDir.replace(parentName + config.osSeparator, '')

                if (data.Children != 0) {
                    return
                }

                if (fs.existsSync(path.join(scriptDir, config.source + '.server' + config.extension))) {
                    fs.renameSync(path.join(scriptDir, config.source + '.server' + config.extension), path.join(parentDir, parentName + '.server' + config.extension))
                    fs.rmdirSync(path.join(parentDir, parentName))
                }
                else if (fs.existsSync(path.join(scriptDir, config.source + '.client' + config.extension))) {
                    fs.renameSync(path.join(scriptDir, config.source + '.client' + config.extension), path.join(parentDir, parentName + '.client' + config.extension))
                    fs.rmdirSync(path.join(parentDir, parentName))
                }
                else if (fs.existsSync(path.join(scriptDir, config.source + config.extension))) {
                    fs.renameSync(path.join(scriptDir, config.source + config.extension), path.join(parentDir, parentName + config.extension))
                    fs.rmdirSync(path.join(parentDir, parentName))
                }

                break
            case 'convert':
                let newDir = files.applyCustomPaths(path.join(rootDir, data.NewPath))

                if (!data.Undo) {
                    let oldDir = files.applyCustomPaths(path.join(rootDir, data.OldPath + config.extension))

                    if (!fs.existsSync(newDir)) {
                        fs.mkdirSync(newDir)
                    }

                    if (fs.existsSync(oldDir)) {
                        let suffix = data.Type
    
                        switch (suffix) {
                            case 'Script':
                                suffix = config.source + '.server'
                                break
                            case 'LocalScript':
                                suffix = config.source + '.client'
                                break
                            case 'ModuleScript':
                                suffix = config.source
                                break
                        }

                        newDir = path.join(newDir, suffix + config.extension)
                        fs.renameSync(oldDir, newDir)
                    }
                }
                else {
                    let oldDir = files.applyCustomPaths(path.join(rootDir, data.OldPath))

                    if (fs.existsSync(oldDir)) {
                        let suffix = data.Type
                        let parentDir = oldDir

                        switch (suffix) {
                            case 'Script':
                                suffix = config.source + '.server'
                                break
                            case 'LocalScript':
                                suffix = config.source + '.client'
                                break
                            case 'ModuleScript':
                                suffix = config.source
                                break
                        }

                        oldDir = path.join(oldDir, suffix + config.extension)

                        if (fs.existsSync(oldDir)) {
                            fs.renameSync(oldDir, newDir + config.extension)
                        }

                        if (fs.existsSync(parentDir)) {
                            setTimeout(() => {
                                fs.rmdirSync(parentDir)
                            }, 100)
                        }
                    }
                }
                break
        }
    }
}

module.exports = {
    sync
}