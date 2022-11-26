local StudioService = game:GetService('StudioService')

local FileHandler = require(script.Parent.FileHandler)
local Config = require(script.Parent.Config)

local connections = {}
local matrix = {}
local queue = {}
local isSyncing = false

local twoWaySync = {}

local function pathChanged(instance, parent)
    if instance.Parent then
        local name = string.split(instance:GetFullName(), '.')[1]

        if Config.syncedDirectories[name] ~= nil and not Config.syncedDirectories[name] then
            connections[instance]:Disconnect()
            connections[instance] = nil
            matrix[instance] = nil
            return
        end

        if parent and instance.Parent:IsA('LuaSourceContainer') and not matrix[instance.Parent].ScriptParent then
            table.insert(queue, {Action = 'convert', OldPath = matrix[instance].Path, NewPath = FileHandler.getPath(instance.Parent)})
        end

        local path = FileHandler.getPath(instance)
        table.insert(queue, {Action = 'changePath', OldPath = matrix[instance].Path, NewPath = path})
        matrix[instance] = path
    else
        table.insert(queue, {Action = 'remove', Path = matrix[instance].Path})
        connections[instance]:Disconnect()
        connections[instance] = nil
        matrix[instance] = nil
    end
end

local function sourceChanged(instance)
    for i, v in ipairs(queue) do
        if v.Instance == instance then
            queue[i] = nil
        end
    end

    table.insert(queue, {Action = 'update', Type = instance.ClassName, Path = FileHandler.getPath(instance), Source = instance.Source, Instance = instance})
end

local function handleInstance(instance)
    if instance:FindFirstChildWhichIsA('LuaSourceContainer') then
        matrix[instance] = {Path = FileHandler.getPath(instance), {ScriptParent = true}}
    else
        matrix[instance] = {Path = FileHandler.getPath(instance), {ScriptParent = false}}
    end

    connections[instance] = instance:GetPropertyChangedSignal('Name'):Connect(function()
        pathChanged(instance)
    end)

    connections[instance] = instance:GetPropertyChangedSignal('Parent'):Connect(function()
        pathChanged(instance, true)
    end)

    connections[instance] = instance:GetPropertyChangedSignal('Source'):Connect(function()
        sourceChanged(instance)
    end)
end

function twoWaySync.run()
    if not isSyncing then
        isSyncing = true

        for i, v in pairs(Config.syncedDirectories) do
            if v then
                for _, w in ipairs(game:GetService(i):GetDescendants()) do
                    if w:IsA('LuaSourceContainer') then
                        handleInstance(w)
                    end
                end

                connections[i] = game:GetService(i).DescendantAdded:Connect(function(descendant)
                    if descendant:IsA('LuaSourceContainer') then
                        if not matrix[descendant] then
                            handleInstance(descendant)
                        end
                    end
                end)
            end
        end
    end
end

function twoWaySync.stop()
    if isSyncing then
        for _, v in pairs(connections) do
            v:Disconnect()
        end

        print(queue)

        isSyncing = false
        connections = {}
        matrix = {}
        queue = {}
    end
end

function twoWaySync.update()
    if isSyncing then
        twoWaySync.stop()
        task.wait()
        twoWaySync.run()
    end
end

return twoWaySync