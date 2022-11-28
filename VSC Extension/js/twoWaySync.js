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
                var dir = path.join(rootDir, data.Path + config.extension)

                if (fs.existsSync(dir)) {
                    fs.writeFileSync(dir, data.Source)
                }

                break
            case 'changePath':
                console.log(data);
                if (data.Children && data.Children != 0) {
                    var oldDir = path.join(rootDir, data.OldPath)

                    if (fs.existsSync(oldDir)) {
                        fs.renameSync(oldDir, path.join(rootDir, data.NewPath))
                    }
                }
                else {
                    var oldDir = path.join(rootDir, data.OldPath + config.extension)

                    if (fs.existsSync(oldDir)) {
                        fs.renameSync(oldDir, path.join(rootDir, data.NewPath + config.extension))
                    }
                    else if (data.Source) {
                        fs.writeFileSync(path.join(rootDir, data.NewPath + config.extension), data.Source)
                    }
                }
                break
            case 'remove':
                var dir = path.join(rootDir, data.Path + config.extension)

                if (fs.existsSync(dir)) {
                    fs.rmSync(dir)
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