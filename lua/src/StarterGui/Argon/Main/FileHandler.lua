local SCRIPT_TYPES = {'LocalScript', 'ModuleScript', 'Script'}

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
    local success, response = pcall(function()
        local object = Instance.new(type)
        object.Name = name
        object.Parent = getInstance(parent)
    end)

    if not success then
        warn('Argon: '..response)
    end
end

function fileHandler.update(object, source)
    local success, response = pcall(function()
        getInstance(object).Source = source
    end)

    if not success then
        warn('Argon: '..response)
    end
end

function fileHandler.delete(object)
    local success, response = pcall(function()
        getInstance(object):Destroy()
    end)

    if not success then
        warn('Argon: '..response)
    end
end

function fileHandler.rename(object, name)
    local success, response = pcall(function()
        getInstance(object).Name = name
    end)

    if not success then
        warn('Argon: '..response)
    end
end

function fileHandler.changeParent(object, parent)
    local success, response = pcall(function()
        getInstance(object).Parent = getInstance(parent)
    end)

    if not success then
        warn('Argon: '..response)
    end
end

function fileHandler.changeType(object, type, name)
    local success, response = pcall(function()
        object = getInstance(object)

        local newObject = Instance.new(type)
        newObject.Parent = object.Parent
        newObject.Name = name or object.Name

        for _, v in ipairs(object:GetChildren()) do
            v.Parent = newObject
        end

        if table.find(SCRIPT_TYPES, type) and table.find(SCRIPT_TYPES, object.ClassName) then
            newObject.Source = object.Source
        end

        object:Destroy()
    end)

    if not success then
        warn('Argon: '..response)
    end
end

return fileHandler