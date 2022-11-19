local HttpService = game:GetService('HttpService')

local FileHandler = require(script.Parent.FileHandler)
local Data = require(script.Parent.Data)

local URL = 'http://%s:%s/'

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
    local headers = {
        action = 'getSync'
    }

    thread = task.spawn(function()
        local success, response = pcall(function()
            while task.wait(0.2) do
                local queue = HttpService:JSONDecode(HttpService:GetAsync(url, false, headers))

                for _, v in ipairs(queue) do
                    if v.Action == 'create' then
                        FileHandler.create(v.Type, v.Name, v.Parent, v.Delete)
                    elseif v.Action == 'update' then
                        FileHandler.update(v.Object, v.Source)
                    elseif v.Action == 'delete' then
                        FileHandler.delete(v.Object)
                    elseif v.Action == 'rename' then
                        FileHandler.rename(v.Object, v.Name)
                    elseif v.Action == 'changeParent' then
                        FileHandler.changeParent(v.Object, v.Parent)
                    elseif v.Action == 'changeType' then
                        FileHandler.changeType(v.Object, v.Type, v.Name)
                    end
                end
            end
        end)

        if not success then
            func(response)
        end
    end)
end

function httpHandler.connect(fail)
    local url = string.format(URL, Data.host, Data.port)
    local headers = {
        action = 'init'
    }

    local success, response = pcall(function()
        HttpService:GetAsync(url, false, headers)
    end)

    func = func or fail

    if success then
        startSyncing(url)
    end

    return success, response
end

function httpHandler.stop()
    if thread then
        task.cancel(thread)
        thread = nil
    end
end

function httpHandler.portInstances(instancesToSync)
    local url = string.format(URL, Data.host, Data.port)
    local headers = {
        action = 'portInstances'
    }

    local success, response = pcall(function()
        HttpService:PostAsync(url, HttpService:JSONEncode(instancesToSync), Enum.HttpContentType.ApplicationJson, false, headers)
    end)

    return success, response
end

function httpHandler.portScripts(scriptsToSync)
    if #scriptsToSync == 0 then
        return true
    end

    local url = string.format(URL, Data.host, Data.port)
    local headers = {
        action = 'portScripts'
    }

    local success, response = pcall(function()
        repeat
            task.wait(0.5)
        until tonumber(HttpService:GetAsync(url, false, {action = 'getState'})) > 500

        local chunks = {}
        local index = 1

        while index ~= #scriptsToSync do
            local chunk
            chunk, index = getChunk(scriptsToSync, index)
            table.insert(chunks, chunk)
        end

        for _, v in ipairs(chunks) do
            HttpService:PostAsync(url, HttpService:JSONEncode(v), Enum.HttpContentType.ApplicationJson, false, headers)

            repeat
                task.wait(0.5)
            until tonumber(HttpService:GetAsync(url, false, {action = 'getState'})) > 500
        end
    end)

    return success, response
end

function httpHandler.portProject()
    local url = string.format(URL, Data.host, Data.port)
    local headers = {
        action = 'portProject'
    }

    local success, response = pcall(function()
        local json = HttpService:JSONDecode(HttpService:GetAsync(url, false, headers))
        local project, length = json.Project, json.Length

        for _, v in ipairs(project) do
            FileHandler.create(v.Type, v.Name, v.Parent, v.Delete)
        end

        repeat
            local chunk
            json = HttpService:JSONDecode(HttpService:GetAsync(url, false, {action = 'portProjectSource'}))
            chunk, length = json.Chunk, json.Length

            for _, v in ipairs(chunk) do
                FileHandler.update(v.Object, v.Source)
            end
        until length <= 0
    end)

    return success, response
end

return httpHandler