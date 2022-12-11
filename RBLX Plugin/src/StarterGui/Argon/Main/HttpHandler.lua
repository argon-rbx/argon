local HttpService = game:GetService('HttpService')
local StudioService = game:GetService('StudioService')

local FileHandler = require(script.Parent.FileHandler)
local TwoWaySync = require(script.Parent.TwoWaySync)
local Config = require(script.Parent.Config)

local API_URL = 'https://dervexhero.github.io/Argon/'
local URL = 'http://%s:%s/'
local SYNC_INTERVAL = 0.2

local thread = nil
local func = nil

local httpHandler = {}

local function getChunk(data, index)
    local chunk, lastChunk = {}, {}

    for i = index, #data do
        index = i
        table.insert(chunk, data[i])

        if #HttpService:JSONEncode(chunk) / 1000 < 1020 then
            table.insert(lastChunk, data[i])
        else
            return lastChunk, index
        end
    end

    return lastChunk, index
end

local function startSyncing(url)
    local getHeader = {action = 'getSync'}
    local setHeader = {action = 'setSync'}

    thread = task.spawn(function()
        local success, response = pcall(function()
            while task.wait(SYNC_INTERVAL) do
                local queue = HttpService:JSONDecode(HttpService:GetAsync(url, false, getHeader))

                for _, v in ipairs(queue) do
                    if v.Action == 'update' then
                        FileHandler.update(v.Object, v.Source)
                    elseif v.Action == 'create' then
                        FileHandler.create(v.Type, v.Name, v.Parent, v.Delete)
                    elseif v.Action == 'delete' then
                        FileHandler.delete(v.Object)
                    elseif v.Action == 'rename' then
                        FileHandler.rename(v.Object, v.Name)
                    elseif v.Action == 'changeParent' then
                        FileHandler.changeParent(v.Object, v.Parent)
                    elseif v.Action == 'changeType' then
                        FileHandler.changeType(v.Object, v.Type, v.Name)
                    elseif v.Action == 'setProperties' then
                        FileHandler.setProperties(v.Object, v.Properties)
                    elseif v.Action == 'closeFile' then
                        httpHandler.openFile()
                    end
                end

                if Config.twoWaySync and #TwoWaySync.queue ~= 0 then
                    HttpService:PostAsync(url, HttpService:JSONEncode(TwoWaySync.queue), Enum.HttpContentType.ApplicationJson, false, setHeader)
                    TwoWaySync.queue = {}
                end
            end
        end)

        if not success then
            func(response)
        end
    end)
end

function httpHandler.connect(fail)
    local url = URL:format(Config.host, Config.port)
    local header = {action = 'init'}

    local success, response = pcall(function()
        if HttpService:GetAsync(url, false, header) == 'true' then
            error('Argon is already connected!', 0)
        end
    end)

    func = func or fail

    if success then
        startSyncing(url)
    end

    return success, response
end

function httpHandler.disconnect()
    if thread then
        task.cancel(thread)
        thread = nil
    end
end

function httpHandler.checkForUpdates()
    local update = nil

    pcall(function()
        local json = HttpService:JSONDecode(HttpService:GetAsync(API_URL))

        if json.plugin ~= Config.argonVersion then
            update = json.plugin
        end
    end)

    return update
end

function httpHandler.openFile(file)
    if file then
        local url = URL:format(Config.host, Config.port)
        local header = {action = 'openFile'}

        pcall(function()
            HttpService:PostAsync(url, HttpService:JSONEncode(file), Enum.HttpContentType.ApplicationJson, false, header)
        end)
    else
        local activeScript = StudioService.ActiveScript

        if activeScript then
            TwoWaySync.pause()
            local newScript = Instance.new(activeScript.ClassName, activeScript.Parent)

            newScript.Source = activeScript.Source
            newScript.Name = activeScript.Name

            for _, v in ipairs(activeScript:GetChildren()) do
                v.Parent = newScript
            end

            if newScript:IsA('Script') then
                newScript.Enabled = activeScript.Enabled
                newScript.RunContext = activeScript.RunContext
            elseif newScript:IsA('LocalScript') then
                newScript.Enabled = activeScript.Enabled
            end

            activeScript:Destroy()
            TwoWaySync.resume()
        end
    end
end

function httpHandler.portInstances(instancesToSync)
    local url = URL:format(Config.host, Config.port)
    local header = {action = 'portInstances'}

    local body = {
        mode = Config.onlyCode,
        instances = instancesToSync
    }

    local success, response = pcall(function()
        HttpService:PostAsync(url, HttpService:JSONEncode(body), Enum.HttpContentType.ApplicationJson, false, header)
    end)

    return success, response
end

function httpHandler.portScripts(scriptsToSync)
    if #scriptsToSync == 0 then
        return true
    end

    local url = URL:format(Config.host, Config.port)
    local portHeader = {action = 'portScripts'}
    local stateHeader = {action = 'getState'}

    local success, response = pcall(function()
        while tonumber(HttpService:GetAsync(url, false, stateHeader)) < 200 do
            task.wait(0.2)
        end

        local chunks = {}
        local index = 1

        repeat
            local chunk
            chunk, index = getChunk(scriptsToSync, index)
            table.insert(chunks, chunk)
        until index == #scriptsToSync

        for _, v in ipairs(chunks) do
            HttpService:PostAsync(url, HttpService:JSONEncode(v), Enum.HttpContentType.ApplicationJson, false, portHeader)

            while tonumber(HttpService:GetAsync(url, false, stateHeader)) < 200 do
                task.wait(0.2)
            end
        end
    end)

    return success, response
end

function httpHandler.portProperties(propertiesToSync)
    local url = URL:format(Config.host, Config.port)
    local header = {action = 'portProperties'}

    local success, response = pcall(function()
        HttpService:PostAsync(url, HttpService:JSONEncode(propertiesToSync), Enum.HttpContentType.ApplicationJson, false, header)
    end)

    return success, response
end

function httpHandler.portProject()
    local url = URL:format(Config.host, Config.port)
    local projectHeader = {action = 'portProject'}
    local sourceHeader = {action = 'portProjectSource'}

    local success, response = pcall(function()
        local json = HttpService:JSONDecode(HttpService:GetAsync(url, false, projectHeader))
        local project, length = json.Project, json.Length

        for _, v in ipairs(project) do
            if v.Action == 'create' then
                FileHandler.create(v.Type, v.Name, v.Parent, v.Delete)
            elseif v.Action == 'setProperties' then
                FileHandler.setProperties(v.Object, v.Properties)
            end
        end

        repeat
            local chunk
            json = HttpService:JSONDecode(HttpService:GetAsync(url, false, sourceHeader))
            chunk, length = json.Chunk, json.Length

            for _, v in ipairs(chunk) do
                FileHandler.update(v.Object, v.Source)
            end
        until length <= 0
    end)

    return success, response
end

return httpHandler