local HttpService = game:GetService('HttpService')
local MarketPlaceService = game:GetService('MarketplaceService')

local FileHandler = require(script.Parent.FileHandler)
local TwoWaySync = require(script.Parent.TwoWaySync)
local Config = require(script.Parent.Config)

local CONTENT_TYPE = Enum.HttpContentType.ApplicationJson
local API_URL = 'https://dervexhero.github.io/Argon/'
local URL = 'http://%s:%s/'
local SYNC_INTERVAL = 0.2

local thread = nil
local snippetThread = nil
local widget = nil
local func = nil

local httpHandler = {}
httpHandler.file = nil

local function syncTitle(title)
    local url = URL:format(Config.host, Config.port)
    local header = {action = 'syncTitle'}

    widget.Title = 'Argon - '..title

    if game.Name:find('.rbxl') or game.Name:find('.rbxlx') then
        title = game.Name
    else
        pcall(function()
            title = MarketPlaceService:GetProductInfo(game.PlaceId).Name
        end)
    end

    HttpService:PostAsync(url, title, CONTENT_TYPE, false, header)
end

local function executeSnippet(snippet)
    if snippetThread then
        task.cancel(snippetThread)
        snippetThread = nil
    end

    snippetThread = task.spawn(function()
        local success, response = pcall(function()
            loadstring(snippet)()
            snippetThread = nil
        end)

        if not success then
            response = response:split('"]:')
            warn(response[#response])
        end
    end)
end

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
                        FileHandler.setProperties(v.Object, HttpService:JSONDecode(v.Properties))
                    elseif v.Action == 'closeFile' then
                        httpHandler.openFile()
                    elseif v.Action == 'executeSnippet' then
                        executeSnippet(v.Snippet)
                    elseif v.Action == 'syncTitle' then
                        syncTitle(v.Title)
                    end
                end

                if Config.twoWaySync and #TwoWaySync.queue ~= 0 then
                    HttpService:PostAsync(url, HttpService:JSONEncode(TwoWaySync.queue), CONTENT_TYPE, false, setHeader)
                    TwoWaySync.queue = {}
                end
            end
        end)

        if not success then
            func(response)
        end
    end)
end

function httpHandler.connect(newWidget, fail)
    widget = widget or newWidget
    func = func or fail

    local url = URL:format(Config.host, Config.port)
    local header = {action = 'init'}

    local success, response = pcall(function()
        local json = HttpService:JSONDecode(HttpService:GetAsync(url, false, header))

        if json.State then
            error('Argon is already connected!', 0)
        end

        if json.Version ~= Config.argonVersion then
            warn('Argon: detected version mismatch! Please update both plugin and extension to avoid errors!')
        end

        if json.Separator then
            Config.separator = json.Separator
        end

        if json.Title then
            syncTitle(json.Title)
        end
    end)

    if success then
        startSyncing(url)
    end

    return success, response
end

function httpHandler.disconnect()
    local url = URL:format(Config.host, Config.port)
    local header = {action = 'disconnect'}

    if thread then
        task.cancel(thread)
        thread = nil

        HttpService:PostAsync(url, HttpService:JSONEncode(''), CONTENT_TYPE, false, header)
    end
end

function httpHandler.checkForUpdates()
    local update = nil

    pcall(function()
        local json = HttpService:JSONDecode(HttpService:GetAsync(API_URL))

        if json.version ~= Config.argonVersion then
            update = json.version
        end
    end)

    return update
end

function httpHandler.openFile(file)
    if file then
        local url = URL:format(Config.host, Config.port)
        local header = {action = 'openFile'}

        pcall(function()
            HttpService:PostAsync(url, HttpService:JSONEncode(file), CONTENT_TYPE, false, header)
        end)
    elseif httpHandler.file then
        httpHandler.file:CloseAsync()
        httpHandler.file = nil
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
        HttpService:PostAsync(url, HttpService:JSONEncode(body), CONTENT_TYPE, false, header)
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
    local clearHeader = {action = 'clearFolders'}

    local success, response = pcall(function()
        while tonumber(HttpService:GetAsync(url, false, stateHeader)) < 100 do
            task.wait(0.1)
        end

        local chunks = {}
        local index = 1

        repeat
            local chunk
            chunk, index = getChunk(scriptsToSync, index)
            table.insert(chunks, chunk)
        until index == #scriptsToSync

        for _, v in ipairs(chunks) do
            HttpService:PostAsync(url, HttpService:JSONEncode(v), CONTENT_TYPE, false, portHeader)

            while tonumber(HttpService:GetAsync(url, false, stateHeader)) < 100 do
                task.wait(0.1)
            end
        end

        HttpService:PostAsync(url, '', CONTENT_TYPE, false, clearHeader)
    end)

    return success, response
end

function httpHandler.portProperties(propertiesToSync)
    local url = URL:format(Config.host, Config.port)
    local portHeader = {action = 'portProperties'}
    local stateHeader = {action = 'getState'}

    local success, response = pcall(function()
        while tonumber(HttpService:GetAsync(url, false, stateHeader)) < 100 do
            task.wait(0.1)
        end

        HttpService:PostAsync(url, HttpService:JSONEncode(propertiesToSync), CONTENT_TYPE, false, portHeader)
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
                FileHandler.setProperties(v.Object, HttpService:JSONDecode(v.Properties))
            elseif v.Action == 'lockPackages' then
                FileHandler.lockPackages()
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