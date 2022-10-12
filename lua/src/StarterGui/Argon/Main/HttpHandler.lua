local HttpService = game:GetService('HttpService')
local SoundService = game:GetService('SoundService')

local FileHandler = require(script.Parent.FileHandler)

local URL = 'http://%s:%s/'

local thread = nil
local func = nil

local function startSyncing(url)
    local headers = {
        action = 'getSync'
    }

    thread = task.spawn(function()
        local success, response = pcall(function()
            while task.wait(0.1) do
                local queue = HttpService:JSONDecode(HttpService:GetAsync(url, false, headers))

                for _, v in ipairs(queue) do
                    if v.Action == 'create' then
                        FileHandler.create(v.Type, v.Name, v.Parent)
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

local httpHandler = {}

function httpHandler.connect(host, port, fail)
    local url = string.format(URL, host, port)
    local headers = {
        action = 'test'
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

return httpHandler