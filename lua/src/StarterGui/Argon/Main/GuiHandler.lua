local TweenService = game:GetService('TweenService')

local HttpHandler = require(script.Parent.HttpHandler)

local TWEEN_INFO = TweenInfo.new(0.2, Enum.EasingStyle.Sine, Enum.EasingDirection.InOut)
local SETTINGS_TWEEN_INFO = TweenInfo.new(0.05, Enum.EasingStyle.Sine, Enum.EasingDirection.InOut)
local LOADING_TWEEN_INFO = TweenInfo.new(1, Enum.EasingStyle.Linear, Enum.EasingDirection.InOut, -1)

local BLACK = Color3.fromRGB(0, 0, 0)
local WHITE = Color3.fromRGB(255, 255, 255)

local LIGHT_BLACK = Color3.fromRGB(40, 40, 40)
local LIGHT_WHITE = Color3.fromRGB(240, 240, 240)

local background = script.Parent.Parent.ArgonGui.Root.Background

local mainPage = background.Main
local settingsPage = background.Settings
local aboutPage = background.About

local inputFrame = mainPage.Body.Input
local previewFrame = mainPage.Body.Preview

local connectButton = mainPage.Body.Connect
local hostInput = inputFrame.Host
local portInput = inputFrame.Port
local settingsButton = mainPage.Body.Settings
local aboutButton = mainPage.Body.About

local info = previewFrame.Info
local loading = connectButton.Loading
local action = connectButton.Action

local aboutBack = aboutPage.Header.Back
local settingsBack = settingsPage.Header.Back
local autoRunButton = settingsPage.Body.AutoRun.Button

local host = 'localhost'
local port = '8000'

local autoRun = false

local plugin = nil
local connections = {}
local debounce = false
local state = 0

local function fail(response)
    action.Text = 'PROCEED'
    info.Text = response
    state = 2
    debounce = false
end

local function connect()
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
                action.Text = 'PROCEED'
                info.Text = response
                state = 2
            end
        else
            if state == 1 then
                HttpHandler.stop()
            end

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
    else
        portInput.Text = portInput.Text:gsub('[^%d]', '')

        if #portInput.Text > 5 then
            portInput.Text = portInput.Text:sub(0, -2)
        end
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
    if page1 then
        page1.ZIndex = 1
        page2.ZIndex = 0
    end

    TweenService:Create(mainPage, TWEEN_INFO, {Position = UDim2.fromScale(position, 0)}):Play()
end

local function toggleSetting(setting)
    if setting == 0 then
        autoRun = not autoRun
        plugin:SetSetting('AutoRun', autoRun)

        if autoRun then
            TweenService:Create(autoRunButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(autoRunButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    end
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
end

function guiHandler.run()
    connections[1] = connectButton.MouseButton1Click:Connect(connect)

    connections[2] = hostInput:GetPropertyChangedSignal('Text'):Connect(function() filterInput(0) end)
    connections[3] = portInput:GetPropertyChangedSignal('Text'):Connect(function() filterInput(1) end)
    connections[4] = hostInput.FocusLost:Connect(function() setAddress(hostInput, true) end)
    connections[5] = portInput.FocusLost:Connect(function() setAddress(portInput) end)

    connections[6] = settingsButton.MouseButton1Click:Connect(function() changePage(-1.05, settingsPage, aboutPage) end)
    connections[7] = aboutButton.MouseButton1Click:Connect(function() changePage(1.05, aboutPage, settingsPage) end)
    connections[8] = settingsBack.MouseButton1Click:Connect(function() changePage(0) end)
    connections[9] = aboutBack.MouseButton1Click:Connect(function() changePage(0) end)

    connections[10] = autoRunButton.MouseButton1Click:Connect(function() toggleSetting(0) end)
end

function guiHandler.stop()
    for _, v in pairs(connections) do
        v:Disconnect()
    end

    connections = {}
end

return guiHandler