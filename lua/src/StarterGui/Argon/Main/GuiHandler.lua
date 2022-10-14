local StudioService = game:GetService('StudioService')
local TweenService = game:GetService('TweenService')

local HttpHandler = require(script.Parent.HttpHandler)
local FileHandler = require(script.Parent.FileHandler)
local Data = require(script.Parent.Data)

local TWEEN_INFO = TweenInfo.new(0.2, Enum.EasingStyle.Sine, Enum.EasingDirection.InOut)
local SETTINGS_TWEEN_INFO = TweenInfo.new(0.1, Enum.EasingStyle.Sine, Enum.EasingDirection.InOut)
local LOADING_TWEEN_INFO = TweenInfo.new(1, Enum.EasingStyle.Linear, Enum.EasingDirection.InOut, -1)

local BLACK = Color3.fromRGB(0, 0, 0)
local WHITE = Color3.fromRGB(255, 255, 255)

local LIGHT_BLACK = Color3.fromRGB(40, 40, 40)
local LIGHT_WHITE = Color3.fromRGB(240, 240, 240)

local background = script.Parent.Parent.ArgonGui.Root.Background

local mainPage = background.Main
local settingsPage = background.Settings
local toolsPage = background.Tools

local inputFrame = mainPage.Body.Input
local previewFrame = mainPage.Body.Preview

local connectButton = mainPage.Body.Connect
local hostInput = inputFrame.Host
local portInput = inputFrame.Port
local settingsButton = mainPage.Body.Settings
local toolsButton = mainPage.Body.Tools

local info = previewFrame.Info
local loading = connectButton.Loading
local action = connectButton.Action

local settingsBack = settingsPage.Header.Back
local toolsBack = toolsPage.Header.Back

local autoReconnectButton = settingsPage.Body.AutoReconnect.Button
local autoRunButton = settingsPage.Body.AutoRun.Button
local syncedDirectoriesButton = settingsPage.Body.SyncedDirectories.Button
local ignoredClassesButton = settingsPage.Body.IgnoredClasses.Button

local syncedDirectoriesFrame = settingsPage.SyncedDirectories
local ignoredClassesFrame = settingsPage.IgnoredClasses

local portButton = toolsPage.Body.Port.Button
local updateButton = toolsPage.Body.Update.Button

local host = 'localhost'
local port = '8000'

local autoRun = false
local autoReconnect = false

local plugin = nil
local connections = {}
local subConnections = {}
local expandedSetting = nil
local isPorting = false
local debounce = false
local stopped = false
local state = 0
local connect

local function fail(response)
    action.Text = 'PROCEED'
    info.Text = response
    state = 2
    debounce = false
    stopped = false

    if autoReconnect then
        task.wait(2)

        if not stopped then
            state = 0
            connect()
        end
    end
end

function connect()
    if not debounce then
        debounce = true

        if state == 0 then
            info.Text = 'Connecting...'
            inputFrame.Visible = false
            previewFrame.Visible = true
            action.Visible = false
            loading.Visible = true

            local tween = TweenService:Create(loading, LOADING_TWEEN_INFO, {Rotation = -360})
            tween:Play()

            local success, response = HttpHandler.connect(host, port, fail)

            action.Visible = true
            loading.Visible = false
            loading.Rotation = 0
            tween:Cancel()

            if success then
                action.Text = 'STOP'
                info.Text = host..':'..port
                state = 1
            else
                fail(response)
            end
        else
            if state == 1 then
                HttpHandler.stop()
            end

            stopped = true
            action.Text = 'CONNECT'
            info.Text = 'Connecting...'
            inputFrame.Visible = true
            previewFrame.Visible = false
            state = 0
        end

        debounce = false
    end
end

local function filterInput(input)
    if input == 0 then
        hostInput.Text = hostInput.Text:gsub('[^%a]', '')
    elseif input == 1 then
        portInput.Text = portInput.Text:gsub('[^%d]', '')

        if #portInput.Text > 5 then
            portInput.Text = portInput.Text:sub(0, -2)
        end
    else
        ignoredClassesFrame.Input.Text = ignoredClassesFrame.Input.Text:gsub('[^%a%, ]', '')
    end
end

local function setAddress(input, isHost)
    if isHost then
        host = input.Text

        if host == '' then
            host = 'localhost'
        end

        plugin:SetSetting('Host', host)
    else
        port = input.Text

        if port == '' then
            port =  '8000'
        end

        plugin:SetSetting('Port', port)
    end
end

local function changePage(position, page1, page2)
    if not expandedSetting then
        if page1 then
            page1.ZIndex = 1
            page2.ZIndex = 0
        end

        TweenService:Create(mainPage, TWEEN_INFO, {Position = UDim2.fromScale(position, 0)}):Play()
    else
        TweenService:Create(settingsPage[expandedSetting], TWEEN_INFO, {Position = UDim2.fromScale(1.05, 0)}):Play()
        TweenService:Create(settingsPage.Body, TWEEN_INFO, {Position = UDim2.fromScale(0, 0)}):Play()
        expandedSetting = nil

        for _, v in pairs(subConnections) do
            v:Disconnect()
        end

        subConnections = {}
    end
end

local function toggleSetting(setting, data)
    if setting == 0 then
        autoRun = not autoRun
        plugin:SetSetting('AutoRun', autoRun)

        if autoRun then
            TweenService:Create(autoRunButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(autoRunButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    elseif setting == 1 then
        autoReconnect = not autoReconnect
        plugin:SetSetting('AutoReconnect', autoReconnect)

        if autoReconnect then
            TweenService:Create(autoReconnectButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(autoReconnectButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    elseif setting == 2 then
        data = data:gsub(' ', '')
        data = string.split(data, '.')

        Data.ignoredClasses = data
        plugin:SetSetting('IgnoredClasses', data)
    else
        local syncState = not Data.syncedDirectories[setting]
        Data.syncedDirectories[setting] = syncState
        plugin:SetSetting('SyncedDirectories', Data.syncedDirectories)

        if syncState then
            TweenService:Create(data.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(data.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    end
end

local function expandSetting(setting)
    expandedSetting = setting
    TweenService:Create(settingsPage[setting], TWEEN_INFO, {Position = UDim2.fromScale(0, 0)}):Play()
    TweenService:Create(settingsPage.Body, TWEEN_INFO, {Position = UDim2.fromScale(-1.05, 0)}):Play()

    for _, v in ipairs(settingsPage[setting]:GetDescendants()) do
        if v:IsA('ImageButton') then
            subConnections[v.Parent.Name] = v.MouseButton1Click:Connect(function()
                toggleSetting(v.Parent.Name, v)
            end)
        elseif v:IsA('TextBox') then
            subConnections[v.Parent.Name] = v:GetPropertyChangedSignal('Text'):Connect(function()
                filterInput(2)
            end)
            subConnections[v.Parent.Name..2] = v.FocusLost:Connect(function()
                toggleSetting(2, v.Text)
            end)
        end
    end
end

local function portToVS()
    
end

local guiHandler = {}

function guiHandler.init(newPlugin)
    local theme = settings().Studio.Theme.Name

    if theme == 'Dark' then
        background.BackgroundColor3 = Color3.fromHex('#2e2e2e')

        for _, v in ipairs(background:GetDescendants()) do
            if v:IsA('ImageButton') or v.Name == 'Input' then
                v.BackgroundColor3 = WHITE
            elseif v:IsA('TextBox') or (v:IsA('TextLabel') and v.Name ~= 'Text') then
                v.TextColor3 = LIGHT_WHITE
            elseif v.Name == 'Icon' then
                v.ImageColor3 = LIGHT_WHITE
            end
        end
    elseif theme == 'Light' then
        background.BackgroundColor3 = Color3.fromHex('#ffffff')

        for _, v in ipairs(background:GetDescendants()) do
            if v:IsA('ImageButton') or v.Name == 'Input' then
                v.BackgroundColor3 = BLACK
            elseif v:IsA('TextBox') or (v:IsA('TextLabel') and v.Name ~= 'Text') then
                v.TextColor3 = LIGHT_BLACK
            elseif v.Name == 'Icon' then
                v.ImageColor3 = LIGHT_BLACK
            end
        end
    end

    changePage(0)
    plugin = newPlugin

    local hostSetting = plugin:GetSetting('Host')
    local portSetting = plugin:GetSetting('Port')
    local autoRunSetting = plugin:GetSetting('AutoRun')
    local autoReconnectSetting = plugin:GetSetting('AutoReconnect')
    local syncedDirectoriesSetting = plugin:GetSetting('SyncedDirectories')
    local ignoredClassesSetting = plugin:GetSetting('IgnoredClasses')

    if hostSetting and hostSetting ~= host then
        hostInput.Text = hostSetting
        host = hostSetting
    end

    if portSetting and portSetting ~= port then
        portInput.Text = portSetting
        port = portSetting
    end

    if autoRunSetting and autoRunSetting ~= autoRun then
        autoRun = autoRunSetting

        if autoRun then
            autoRunButton.OnIcon.ImageTransparency = 0
        end
    end

    if autoReconnectSetting and autoReconnectSetting ~= autoReconnect then
        autoReconnect = autoReconnectSetting

        if autoReconnect then
            autoReconnectButton.OnIcon.ImageTransparency = 0
        end
    end

    Data.syncedDirectories = syncedDirectoriesSetting or Data.syncedDirectories
    for i, v in pairs(Data.syncedDirectories) do
        local properties = StudioService:GetClassIcon(i)
        local icon = syncedDirectoriesFrame[i].ClassIcon

        for j, w in pairs(properties) do
            icon[j] = w
        end

        if v then
            syncedDirectoriesFrame[i].Button.OnIcon.ImageTransparency = 0
        end
    end

    Data.ignoredClasses = ignoredClassesSetting or Data.ignoredClasses
    if ignoredClassesSetting then
        local text = ''

        for i, v in ipairs(Data.ignoredClasses) do
            if i ~= 1 then
                text = text..', '..v
            else
                text = v
            end
        end

        ignoredClassesFrame.Input.Text = text
    end
end

function guiHandler.run(autoConnect)
    connections['connectButton'] = connectButton.MouseButton1Click:Connect(connect)

    connections['hostInput'] = hostInput:GetPropertyChangedSignal('Text'):Connect(function() filterInput(0) end)
    connections['portInput'] = portInput:GetPropertyChangedSignal('Text'):Connect(function() filterInput(1) end)
    connections['hostInput2'] = hostInput.FocusLost:Connect(function() setAddress(hostInput, true) end)
    connections['portInput2'] = portInput.FocusLost:Connect(function() setAddress(portInput) end)

    connections['settingsButton'] = settingsButton.MouseButton1Click:Connect(function() changePage(-1.05, settingsPage, toolsPage) end)
    connections['toolsButton'] = toolsButton.MouseButton1Click:Connect(function() changePage(1.05, toolsPage, settingsPage) end)
    connections['settingsBack'] = settingsBack.MouseButton1Click:Connect(function() changePage(0) end)
    connections['toolsBack'] = toolsBack.MouseButton1Click:Connect(function() changePage(0) end)

    connections['autoRunButton'] = autoRunButton.MouseButton1Click:Connect(function() toggleSetting(0) end)
    connections['autoReconnectButton'] = autoReconnectButton.MouseButton1Click:Connect(function() toggleSetting(1) end)
    connections['syncedDirectoriesButton'] = syncedDirectoriesButton.MouseButton1Click:Connect(function() expandSetting('SyncedDirectories') end)
    connections['ignoredClassesButton'] = ignoredClassesButton.MouseButton1Click:Connect(function() expandSetting('IgnoredClasses') end)

    if autoConnect then
        connect()
    end
end

function guiHandler.stop()
    for _, v in pairs(connections) do
        v:Disconnect()
    end

    connections = {}
end

return guiHandler