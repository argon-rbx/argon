local StudioService = game:GetService('StudioService')

local FileHandler = require(script.Parent.FileHandler)
local Config = require(script.Parent.Config)

local matrix = {}
local connections = {}
local isSyncing = false
local sourceConnection = nil

local twoWaySync = {}

twoWaySync.queue = {}

local function pathChanged(instance, parent, new)
    if instance.Parent then
        local name = instance:GetFullName():split('.')[1]

        if Config.syncedDirectories[name] ~= nil and not Config.syncedDirectories[name] then
            for _, v in pairs(connections[instance]) do
                v:Disconnect()
            end
            connections[instance] = nil
            matrix[instance] = nil
            return
        end

        local path = FileHandler.getPath(instance, true)

        if parent then
            if instance.Parent:IsA('LuaSourceContainer') then
                local newPath = FileHandler.getPath(instance.Parent, true)
                table.insert(twoWaySync.queue, {Action = 'convert', OldPath = matrix[instance.Parent].Path, NewPath = newPath, Type = instance.Parent.ClassName})
                matrix[instance.Parent].Path = newPath
            end

            if matrix[instance].Parent:IsA('LuaSourceContainer') and FileHandler.countChildren(matrix[instance].Parent) == 0 then
                local newPath = FileHandler.getPath(matrix[instance].Parent, true)
                table.insert(twoWaySync.queue, {Action = 'convert', OldPath = matrix[matrix[instance].Parent].Path, NewPath = newPath, Type = matrix[instance].Parent.ClassName, Undo = true})
                matrix[matrix[instance].Parent].Path = newPath
            end

            if new then
                table.insert(twoWaySync.queue, {Action = 'changePath', OldPath = matrix[instance].Path, NewPath = path, Source = instance.Source})
                matrix[instance].Parent = instance.Parent
                matrix[instance].Path = path
                return
            end
        end

        for _, v in ipairs(instance:GetDescendants()) do
            if v:IsA('LuaSourceContainer') then
                matrix[v].Path = FileHandler.getPath(v, true)
            end
        end

        table.insert(twoWaySync.queue, {Action = 'changePath', OldPath = matrix[instance].Path, NewPath = path, Children = FileHandler.countChildren(instance)})
        matrix[instance].Parent = instance.Parent
        matrix[instance].Path = path
    else
        table.insert(twoWaySync.queue, {Action = 'remove', Path = matrix[instance].Path, Children = FileHandler.countChildren(matrix[instance].Parent)})

        if matrix[instance].Parent:IsA('LuaSourceContainer') then
            matrix[matrix[instance].Parent].Path = FileHandler.getPath(matrix[instance].Parent, true)
        end

        for _, v in pairs(connections[instance]) do
            v:Disconnect()
        end
        connections[instance] = nil
        matrix[instance] = nil
    end
end

local function sourceChanged(instance)
    if FileHandler.countChildren(instance) == 0 then
        table.insert(twoWaySync.queue, {Action = 'sync', Path = FileHandler.getPath(instance, true), Source = instance.Source, Instance = instance})
    else
        table.insert(twoWaySync.queue, {Action = 'sync', Type = instance.ClassName, Path = FileHandler.getPath(instance, true), Source = instance.Source, Instance = instance})
    end
end

local function handleInstance(instance, new)
    matrix[instance] = {Path = FileHandler.getPath(instance, true), Parent = instance.Parent}
    connections[instance] = {}

    table.insert(connections[instance], instance:GetPropertyChangedSignal('Name'):Connect(function()
        pathChanged(instance)
    end))

    table.insert(connections[instance], instance:GetPropertyChangedSignal('Parent'):Connect(function()
        pathChanged(instance, true, new)
        new = false
    end))
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
                            handleInstance(descendant, true)
                        end
                    end
                end)
            end
        end

        connections[StudioService] = StudioService.Changed:Connect(function(property)
            if property == 'ActiveScript' then
                local instance = StudioService.ActiveScript

                if instance then
                    if sourceConnection then
                        sourceConnection:Disconnect()
                        sourceConnection = nil
                    end

                    sourceConnection = instance:GetPropertyChangedSignal('Source'):Connect(function()
                        sourceChanged(instance)
                    end)
                elseif sourceConnection then
                    sourceConnection:Disconnect()
                    sourceConnection = nil
                end
            end
        end)
    end
end

function twoWaySync.stop()
    if isSyncing then
        for _, v in pairs(connections) do
            if typeof(v) == 'table' then
                for _, w in ipairs(v) do
                    w:Disconnect()
                end
            else
                v:Disconnect()
            end
        end

        isSyncing = false
        connections = {}
        matrix = {}
        twoWaySync.queue = {}
    end
end

function twoWaySync.update()
    if isSyncing then
        twoWaySync.stop()
        task.wait()
        twoWaySync.run()
    end
end

function twoWaySync.pause()
    twoWaySync.stop()
end

function twoWaySync.resume()
    if Config.twoWaySync then
        twoWaySync.run()
    end
end

return twoWaySync