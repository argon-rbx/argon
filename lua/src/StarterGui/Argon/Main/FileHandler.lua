local Data = require(script.Parent.Data)

local SCRIPT_TYPES = {
    ['ModuleScript'] = '',
    ['Script'] = 'server',
    ['LocalScript'] = 'client', 
}
local SEPARATOR = '|'

local recursiveCount = 0

local function len(array)
    local index = 0

    for _, _ in pairs(array) do
        index += 1
    end

    return index
end

local function parse(instance)
    local name = instance.Name:gsub('[%:%*%?%"%<%>%|]', '')
    local className = ''

    if name:match('^/') or name:match('^\\') then
        name:sub(2)
    end

    if instance.ClassName ~= 'Folder' then
        className = '.'..instance.ClassName
    end

    return name..className
end

local function getChildren(dir)
    local children = {}

    for _, v in pairs(dir:GetChildren()) do
        if not table.find(Data.ignoredClasses, v.ClassName) and v:GetAttribute('ArgonIgnore') == nil then
            if #v:GetChildren() > 0 then
                children[parse(v)] = getChildren(v)
            else
                children[parse(v)] = {}
            end
        end
    end

    return children
end

local function getParent(instance, class)
    local parent = instance.Parent
    local dir = ''

    recursiveCount += 1

    if instance.ClassName ~= class then
        local name

        if instance:IsA('LuaSourceContainer') then
		name = instance.Name
		if recursiveCount == 1 and #instance:GetChildren() == 0 then
			name ..= '.' .. SCRIPT_TYPES[instance.ClassName]
		end
        elseif instance.ClassName == 'Folder' then
            name = instance.Name
        else
            name = instance.Name..'.'..instance.ClassName
        end

        dir = getParent(parent, class)..'\\'..name
    else
        dir = instance.Name
    end

    return dir
end

local function getInstance(parent)
    local lastParent = game
    parent = parent:split(SEPARATOR)

    for _, v in ipairs(parent) do
        lastParent = lastParent[v]
    end

    return lastParent
end

local fileHandler = {}

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

        if SCRIPT_TYPES[type] and object:IsA('LuaSourceContainer') then
            newObject.Source = object.Source
        end

        object:Destroy()
    end)

    if not success then
        warn('Argon: '..response)
    end
end

function fileHandler.portInstances()
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

function fileHandler.portScripts()
    local scriptsToSync = {}

    for i, v in pairs(Data.syncedDirectories) do
        if v then
            for _, w in ipairs(game[i]:GetDescendants()) do
                if w:IsA("LuaSourceContainer") and w:GetAttribute('ArgonIgnore') == nil then
                    recursiveCount = 0
                    table.insert(scriptsToSync, {Type = w.ClassName, Instance = getParent(w, i), Source = w.Source})
                end
            end
        end
    end

    return scriptsToSync
end

return fileHandler
