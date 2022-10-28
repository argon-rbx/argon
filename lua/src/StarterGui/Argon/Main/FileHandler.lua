local ChangeHistoryService = game:GetService('ChangeHistoryService')

local Data = require(script.Parent.Data)

local SEPARATOR = '|'
local ARGON_IGNORE = 'ArgonIgnore'
local SCRIPT_TYPES = {
    Script = 'server',
    LocalScript = 'client',
    ModuleScript = ''
}

local function addWaypoint()
    ChangeHistoryService:SetWaypoint('ArgonSync')
end

local function len(array)
    local index = 0

    for _, _ in pairs(array) do
        index += 1
    end

    return index
end

local function parse(instance)
    local name, num = instance.Name:gsub('[%:%*%?%"%<%>%|]', '')
    local className = ''

    if name:match('^/') or name:match('^\\') then
        name:sub(2)
    end

    if instance.ClassName ~= 'Folder' then
        className = '.'..instance.ClassName
    end

    if num ~= 0 then
        warn('Argon: '..instance:GetFullName()..' contains invalid symbols! (fhP)')
    end

    return name..className
end

local function getChildren(dir)
    local children = {}

    for _, v in pairs(dir:GetChildren()) do
        if not table.find(Data.ignoredClasses, v.ClassName) and v:GetAttribute(ARGON_IGNORE) == nil then
            if #v:GetChildren() > 0 then
                children[parse(v)] = getChildren(v)
            else
                children[parse(v)] = {}
            end
        end
    end

    return children
end

local function getParent(instance, class, recursive)
    local parent = instance.Parent
    local dir = ''

    if instance.ClassName ~= class then
        local name

        if instance:IsA('LuaSourceContainer') then
		    if not recursive then
                name = instance.Name

                if #instance:GetChildren() == 0 then
                    if instance.ClassName ~= 'ModuleScript' then
                        name ..= '.'..SCRIPT_TYPES[instance.ClassName]
                    else
                        name = name
                    end
                else
                    name = name
                end
            else
                name = instance.Name
		    end
        elseif instance.ClassName == 'Folder' then
            name = instance.Name
        else
            name = instance.Name..'.'..instance.ClassName
        end

        dir = getParent(parent, class, true)..'\\'..name
    else
        dir = instance.ClassName
    end

    return dir
end

local function getInstance(parent)
    local lastParent = game
    parent = parent:split(SEPARATOR)

    for _, v in ipairs(parent) do
        if lastParent == game then
            lastParent = game:GetService(v)
        else
            lastParent = lastParent[v]
        end
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
        elseif parent:FindFirstChild(name) then
            return
        end

        object.Name = name
        object.Parent = parent
    end)

    if not success then
        warn('Argon: '..response..' (fhC)')
    end

    addWaypoint()
end

function fileHandler.update(object, source)
    local success, response = pcall(function()
        getInstance(object).Source = source
    end)

    if not success then
        warn('Argon: '..response..' (fhU)')
    end

    addWaypoint()
end

function fileHandler.delete(object)
    local success, response = pcall(function()
        getInstance(object):Destroy()
    end)

    if not success then
        warn('Argon: '..response..' (fhD)')
    end

    addWaypoint()
end

function fileHandler.rename(object, name)
    local success, response = pcall(function()
        getInstance(object).Name = name
    end)

    if not success then
        warn('Argon: '..response..' (fhR)')
    end

    addWaypoint()
end

function fileHandler.changeParent(object, parent)
    local success, response = pcall(function()
        getInstance(object).Parent = getInstance(parent)
    end)

    if not success then
        warn('Argon: '..response..' (fhCP)')
    end

    addWaypoint()
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

        ---@diagnostic disable-next-line: invalid-class-name
        if SCRIPT_TYPES[type] and object:IsA('LuaSourceContainer') then --why the hell Roblox LSP thinks that this is invalid enum?!
            newObject.Source = object.Source
        end

        object:Destroy()
    end)

    if not success then
        warn('Argon: '..response..' (fhCT)')
    end

    addWaypoint()
end

function fileHandler.portInstances()
    local instancesToSync = {}

    for i, v in pairs(Data.syncedDirectories) do
        if v then
            instancesToSync[i] = getChildren(game:GetService(i))
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
            for _, w in ipairs(game:GetService(i):GetDescendants()) do
                if w:IsA('LuaSourceContainer') and w:GetAttribute(ARGON_IGNORE) == nil then
                    table.insert(scriptsToSync, {Type = w.ClassName, Instance = getParent(w, i), Source = w.Source})
                end
            end
        end
    end

    return scriptsToSync
end

return fileHandler