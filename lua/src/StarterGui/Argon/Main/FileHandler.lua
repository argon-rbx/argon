local Data = require(script.Parent.Data)

local function getParent(parent)
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
    object.Parent = getParent(parent)
end

return fileHandler