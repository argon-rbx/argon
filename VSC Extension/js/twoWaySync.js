const fs = require('fs')
const path = require('path')
const files = require('./files')
const config = require('../config/settings.js')

function sync(queue) {
    queue = JSON.parse(queue)
    let rootDir = files.getRootDir()

    for (let data of queue) {
        switch (data.Action) {
            case 'sync':
                if (data.Type) {
                    let name

                    switch (data.Type) {
                        case 'Script':
                            name = '.source.server'
                            break
                        case 'LocalScript':
                            name = '.source.client'
                            break
                        case 'ModuleScript':
                            name = '.source'
                            break
                    }

                    var dir = path.join(rootDir, data.Path + '\\' + name + config.extension)
                }
                else {
                    var dir = path.join(rootDir, data.Path + config.extension)
                }

                console.log(data, dir);

                if (fs.existsSync(dir)) {
                    fs.writeFileSync(dir, data.Source)
                }

                break
            case 'changePath':
                if (data.Children && data.Children != 0) {
                    var dir = path.join(rootDir, data.OldPath)

                    if (fs.existsSync(dir)) {
                        fs.renameSync(dir, path.join(rootDir, data.NewPath))
                    }
                }
                else {
                    var dir = path.join(rootDir, data.OldPath + config.extension)

                    if (fs.existsSync(dir)) {
                        fs.renameSync(dir, path.join(rootDir, data.NewPath + config.extension))
                    }
                    else if (data.Source) {
                        fs.writeFileSync(path.join(rootDir, data.NewPath + config.extension), data.Source)
                    }
                }
                break
            case 'remove':
                var dir = path.join(rootDir, data.Path + config.extension)
                let splitted = dir.split('\\')

                if (fs.existsSync(dir)) {
                    fs.rmSync(dir)
                }

                let parentName = splitted[splitted.length - 2]
                let scriptDir = dir.replace(splitted[splitted.length - 1], '')
                let parentDir = scriptDir.replace(parentName + '\\', '')

                if (fs.existsSync(path.join(scriptDir, '.source.server' + config.extension))) {
                    fs.renameSync(path.join(scriptDir, '.source.server' + config.extension), path.join(parentDir, parentName + '.server' + config.extension))
                    fs.rmdirSync(path.join(parentDir, parentName))
                }
                else if (path.join(scriptDir, '.source.client' + config.extension)) {
                    fs.renameSync(path.join(scriptDir, '.source.client' + config.extension), path.join(parentDir, parentName + '.client' + config.extension))
                    fs.rmdirSync(path.join(parentDir, parentName))
                }
                else if (path.join(scriptDir, '.source' + config.extension)) {
                    fs.renameSync(path.join(scriptDir, '.source' + config.extension), path.join(parentDir, parentName + config.extension))
                    fs.rmdirSync(path.join(parentDir, parentName))
                }

                break
            case 'convert':
                var oldDir = path.join(rootDir, data.OldPath + config.extension)
                let newDir = path.join(rootDir, data.NewPath)

                if (fs.existsSync(newDir) == false) {
                    fs.mkdirSync(newDir)
                }

                if (fs.existsSync(oldDir)) {
                    let suffix = data.Type

                    switch (suffix) {
                        case 'Script':
                            suffix = '.source.server' + config.extension
                            break
                        case 'LocalScript':
                            suffix = '.source.client' + config.extension
                            break
                        case 'ModuleScript':
                            suffix = '.source' + config.extension
                            break
                    }

                    fs.renameSync(oldDir, path.join(newDir, suffix))
                }
                break

        }
    }
}

module.exports = {
    sync
}