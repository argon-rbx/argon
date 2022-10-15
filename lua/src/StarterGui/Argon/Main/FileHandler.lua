local Data = require(script.Parent.Data)

local SCRIPT_TYPES = {'LocalScript', 'ModuleScript', 'Script'}

local function toArgon(instance)
    return instance.Name..'.'..instance.ClassName
end

local function getChildren(dir)
    local children = {}

    for _, v in pairs(dir:GetChildren()) do
        if not table.find(Data.ignoredClasses, v.ClassName) and v:GetAttribute('ArgonIgnore') == nil then
            if #v:GetChildren() > 0 then
                children[toArgon(v)] = getChildren(v)
            else
                children[toArgon(v)] = {}
            end
        end
    end

    return children
end

local function len(array)
    local index = 0

    for _, _ in pairs(array) do
        index += 1
    end

    return index
end

local fileHandler = {}

fileHandler.separator = '|'

local function getInstance(parent)
    local lastParent = game
    parent = string.split(parent, fileHandler.spearator)

    for _, v in ipairs(parent) do
        lastParent = lastParent[v]
    end

    return lastParent
end

function fileHandler.create(type, name, parent, delete)
    local success, response = pcall(function()
        local object = Instance.new(type)
        parent = getInstance(parent)

        if delete and parent:FindFirstChild(name) then
            parent[name]:Destroy()
        end

        object.Name = name
        object.Parent = parent
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

function fileHandler.port()
    local instancesToSync = {}

    for i, v in pairs(Data.syncedDirectories) do
        if v then
            instancesToSync[i] = getChildren(game[i])
        end
    end

    for i, v in pairs(instancesToSync) do
        if len(v) == 0 then
            instancesToSync[i] = nil
        end
    end

    return instancesToSync
end

return fileHandler