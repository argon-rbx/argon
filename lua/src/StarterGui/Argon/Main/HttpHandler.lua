local HttpService = game:GetService('HttpService')

local FileHandler = require(script.Parent.FileHandler)

local URL = 'http://%s:%s/'

local thread = nil

local function startSyncing(url)
    local headers = {
        action = 'getSync'
    }

    thread = task.spawn(function()
        while task.wait(0.1) do
            local queue = HttpService:JSONDecode(HttpService:GetAsync(url, false, headers))

            for _, v in ipairs(queue) do
                if v.Action == 'create' then
                    FileHandler.create(v.Type, v.Name, v.Parent)
                end
            end
        end
    end)
end

local httpHandler = {}

function httpHandler.connect(host, port)
    local url = string.format(URL, host, port)
    local headers = {
        action = 'test'
    }

    local success, response = pcall(function()
        HttpService:GetAsync(url, false, headers)
    end)

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

return httpHandler