local Data = require(script.Parent.Data)

local function getInstance(parent)
    local lastParent = game
    parent = string.split(parent, '.')

    for _, v in ipairs(parent) do
        lastParent = lastParent[v]
    end

    return lastParent
end

local fileHandler = {}

function fileHandler.create(type, name, parent)
    local object = Instance.new(Data.types[type])
    object.Name = name
    object.Parent = getInstance(parent)
end

function fileHandler.update(object, source)
    getInstance(object).Source = source
end

function fileHandler.delete(object)
    getInstance(object):Destroy()
end

return fileHandler