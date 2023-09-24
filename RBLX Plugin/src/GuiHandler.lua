local ScriptEditorService = game:GetService('ScriptEditorService')
local StudioService = game:GetService('StudioService')
local TweenService = game:GetService('TweenService')
local RunService = game:GetService('RunService')
local Studio = settings():GetService('Studio')

local HttpHandler = require(script.Parent.HttpHandler)
local FileHandler = require(script.Parent.FileHandler)
local TwoWaySync = require(script.Parent.TwoWaySync)
local Config = require(script.Parent.Config)

local PAGES_TWEEN_INFO = TweenInfo.new(0.2, Enum.EasingStyle.Quad, Enum.EasingDirection.InOut)
local SETTINGS_TWEEN_INFO = TweenInfo.new(0.1, Enum.EasingStyle.Sine, Enum.EasingDirection.InOut)
local LOADING_TWEEN_INFO = TweenInfo.new(1, Enum.EasingStyle.Linear, Enum.EasingDirection.InOut, -1)

local BLACK = Color3.fromRGB(0, 0, 0)
local WHITE = Color3.fromRGB(255, 255, 255)

local LIGHT_BLACK = Color3.fromRGB(40, 40, 40)
local LIGHT_WHITE = Color3.fromRGB(240, 240, 240)

local ROBLOX_BLACK = Color3.fromRGB(46, 46, 46)
local ROBLOX_WHITE = Color3.fromRGB(255, 255, 255)

local LOADING_ICON = 'rbxassetid://11234420895'
local START_ICON = 'rbxassetid://11272872815'

local CONNECTED_ICON = 'rbxassetid://11964657306'
local DISCONNECTED_ICON = 'rbxassetid://11230142853'

local AUTO_RECONNECT_DELAY = 3

local background = script.Parent.Parent.ArgonGui.Root.Background

local playtestFrame = background.Playtest
local updateFrame = background.Update

local mainPage = background.Main
local settingsPage = background.Settings
local toolsPage = background.Tools

local versionLabel = mainPage.Header.Version
local inputFrame = mainPage.Body.Input
local previewFrame = mainPage.Body.Preview

local connectButton = mainPage.Body.Connect
local hostInput = inputFrame.Host
local portInput = inputFrame.Port
local settingsButton = mainPage.Body.Settings
local toolsButton = mainPage.Body.Tools

local infoLabel = previewFrame.Info
local loadingImage = connectButton.Loading
local actionLabel = connectButton.Action

local settingsBack = settingsPage.Header.Back
local toolsBack = toolsPage.Header.Back

local autoRunButton = settingsPage.Body.AutoRun.Button
local autoReconnectButton = settingsPage.Body.AutoReconnect.Button
local onlyCodeButton = settingsPage.Body.OnlyCode.Button
local openInEditorButton = settingsPage.Body.OpenInEditor.Button
local twoWaySyncButton = settingsPage.Body.TwoWaySync.Button
local propertySyncingButton = settingsPage.Body.PropertySyncing.Button
local syncDuplicatesButton = settingsPage.Body.SyncDuplicates.Button
local classFilteringButton = settingsPage.Body.ClassFiltering.Button
local syncedDirectoriesButton = settingsPage.Body.SyncedDirectories.Button

local classFilteringFrame = settingsPage.ClassFiltering
local syncedDirectoriesFrame = settingsPage.SyncedDirectories

local portToVSButton = toolsPage.Body.PortToVS.Button
local portToRobloxButton = toolsPage.Body.PortToRoblox.Button

local connections = {}
local subConnections = {}
local themeConnection = nil
local documentConnection = nil
local expandedSetting = nil
local lastTheme = 'Dark'
local isPorting = false
local didSetup = false
local debounce = false
local stopped = false
local widget = nil
local button = nil
local state = 0
local connect

local guiHandler = {}

local function setStatusIcon(isConnected)
    if isConnected then
        if button.Icon ~= CONNECTED_ICON then
            button.Icon = CONNECTED_ICON
        end
    else
        if button.Icon ~= DISCONNECTED_ICON then
            button.Icon = DISCONNECTED_ICON
        end
    end
end

local function fail(response)
    actionLabel.Text = 'PROCEED'
    infoLabel.Text = response
    debounce = false
    stopped = false
    state = 2

    widget.Title = 'Argon'
    setStatusIcon()

    if Config.autoReconnect then
        task.wait(AUTO_RECONNECT_DELAY)

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
            infoLabel.Text = 'Connecting...'
            inputFrame.Visible = false
            previewFrame.Visible = true
            actionLabel.Visible = false
            loadingImage.Visible = true

            local tween = TweenService:Create(loadingImage, LOADING_TWEEN_INFO, {Rotation = -360})
            tween:Play()

            local success, response = HttpHandler.connect(widget, fail)

            actionLabel.Visible = true
            loadingImage.Visible = false
            loadingImage.Rotation = 0
            tween:Cancel()

            if success then
                actionLabel.Text = 'STOP'
                infoLabel.Text = Config.host..':'..Config.port
                setStatusIcon(true)
                state = 1
            else
                fail(response)
            end
        else
            if state == 1 then
                HttpHandler.disconnect()
            end

            stopped = true
            actionLabel.Text = 'CONNECT'
            widget.Title = 'Argon'
            infoLabel.Text = 'Connecting...'
            inputFrame.Visible = true
            previewFrame.Visible = false
            setStatusIcon()
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
        classFilteringFrame.Input.Text = classFilteringFrame.Input.Text:gsub('[^%a%, ]', '')
    end
end

local function setAddress(input, isHost)
    if isHost then
        Config.host = input.Text

        if Config.host == '' then
            Config.host = 'localhost'
        end

        plugin:SetSetting('Host', Config.host)
    else
        Config.port = input.Text

        if Config.port == '' then
            Config.port =  '8000'
        end

        plugin:SetSetting('Port', Config.port)
    end
end

local function changePage(position, page1, page2)
    if not expandedSetting then
        if page1 then
            page1.ZIndex = 1
            page2.ZIndex = 0
        end

        guiHandler.runPage(page1 or mainPage)

        TweenService:Create(mainPage, PAGES_TWEEN_INFO, {Position = UDim2.fromScale(position, 0)}):Play()
    else
        TweenService:Create(settingsPage[expandedSetting], PAGES_TWEEN_INFO, {Position = UDim2.fromScale(1.05, 0)}):Play()
        TweenService:Create(settingsPage.Body, PAGES_TWEEN_INFO, {Position = UDim2.fromScale(0, 0)}):Play()
        expandedSetting = nil

        for _, v in pairs(subConnections) do
            v:Disconnect()
        end

        subConnections = {}
    end
end

local function handleDocumentChange()
    if Config.openInEditor then
        if not documentConnection then
            documentConnection = ScriptEditorService.TextDocumentDidOpen:Connect(function(document)
                if document.Name == 'CommandBar' or state ~= 1 then
                    return
                end

                HttpHandler.file = document
                task.wait()

                local container = document:GetScript()
                local file = {
                    File = FileHandler.getPath(container),
                    Line = document:GetSelectionStart()
                }

                if FileHandler.countChildren(container) ~= 0 then
                    file.Type = container.ClassName
                end

                HttpHandler.openFile(file)
            end)
        end
    elseif documentConnection then
        documentConnection:Disconnect()
        documentConnection = nil
    end
end

local function toggleSetting(setting, data)
    if setting == 0 then
        Config.autoRun = not Config.autoRun
        plugin:SetSetting('AutoRun', Config.autoRun)

        if Config.autoRun then
            TweenService:Create(autoRunButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(autoRunButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    elseif setting == 1 then
        Config.autoReconnect = not Config.autoReconnect
        plugin:SetSetting('AutoReconnect', Config.autoReconnect)

        if Config.autoReconnect then
            TweenService:Create(autoReconnectButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(autoReconnectButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    elseif setting == 2 then
        Config.onlyCode = not Config.onlyCode
        plugin:SetSetting('OnlyCode', Config.onlyCode)

        if Config.onlyCode then
            TweenService:Create(onlyCodeButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(onlyCodeButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    elseif setting == 3 then
        Config.openInEditor = not Config.openInEditor
        plugin:SetSetting('OpenInEditor', Config.openInEditor)
        handleDocumentChange()

        if Config.openInEditor then
            TweenService:Create(openInEditorButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(openInEditorButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    elseif setting == 4 then
        Config.twoWaySync = not Config.twoWaySync
        plugin:SetSetting('TwoWaySync', Config.twoWaySync)

        if Config.twoWaySync then
            TweenService:Create(twoWaySyncButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
            TwoWaySync.run()
        else
            TweenService:Create(twoWaySyncButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
            TwoWaySync.stop()
        end
    elseif setting == 5 then
        Config.propertySyncing = not Config.propertySyncing
        plugin:SetSetting('PropertySyncing', Config.propertySyncing)

        if Config.propertySyncing then
            TweenService:Create(propertySyncingButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(propertySyncingButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    elseif setting == 6 then
        Config.syncDuplicates = not Config.syncDuplicates
        plugin:SetSetting('SyncDuplicates', Config.syncDuplicates)

        if Config.syncDuplicates then
            TweenService:Create(syncDuplicatesButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(syncDuplicatesButton.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end
    elseif setting == 7 then
        Config.filteringMode = not Config.filteringMode
        plugin:SetSetting('FilteringMode', Config.filteringMode)

        if Config.filteringMode then
            TweenService:Create(data.Selector, SETTINGS_TWEEN_INFO, {Position = UDim2.fromScale(0.5, 0)}):Play()
        else
            TweenService:Create(data.Selector, SETTINGS_TWEEN_INFO, {Position = UDim2.fromScale(0, 0)}):Play()
        end
    elseif setting == 8 then
        data = data:gsub(' ', '')
        data = data:split(',')

        Config.filteredClasses = data
        plugin:SetSetting('FilteredClasses', data)
    else
        local syncState = not Config.syncedDirectories[setting]
        Config.syncedDirectories[setting] = syncState
        plugin:SetSetting('SyncedDirectories', Config.syncedDirectories)

        if syncState then
            TweenService:Create(data.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 0}):Play()
        else
            TweenService:Create(data.OnIcon, SETTINGS_TWEEN_INFO, {ImageTransparency = 1}):Play()
        end

        TwoWaySync.update()
    end
end

local function expandSetting(setting)
    if setting == 'SyncedDirectories' then
        syncedDirectoriesFrame.CanvasPosition = Vector2.new(0, 0)
    end

    expandedSetting = setting
    TweenService:Create(settingsPage[setting], PAGES_TWEEN_INFO, {Position = UDim2.fromScale(0, 0)}):Play()
    TweenService:Create(settingsPage.Body, PAGES_TWEEN_INFO, {Position = UDim2.fromScale(-1.05, 0)}):Play()

    for _, v in ipairs(settingsPage[setting]:GetDescendants()) do
        if v:IsA('ImageButton') then
            if v.Name ~= 'Mode' then
                subConnections[v.Parent.Name] = v.MouseButton1Click:Connect(function()
                    toggleSetting(v.Parent.Name, v)
                end)
            else
                subConnections[v.Parent.Name] = v.MouseButton1Click:Connect(function()
                    toggleSetting(7, v)
                end)
            end
        elseif v:IsA('TextBox') then
            subConnections[v.Parent.Name] = v:GetPropertyChangedSignal('Text'):Connect(function()
                filterInput(2)
            end)
            subConnections[v.Parent.Name..2] = v.FocusLost:Connect(function()
                toggleSetting(8, v.Text)
            end)
        end
    end
end

local function portToVS()
    if not isPorting and state == 1 then
        isPorting = true

        local tween = TweenService:Create(portToVSButton.Icon, LOADING_TWEEN_INFO, {Rotation = -360})
        portToVSButton.Icon.Position = UDim2.fromScale(0.5, 0.5)
        portToVSButton.Icon.Image = LOADING_ICON
        tween:Play()

        local success, response = HttpHandler.portInstances(FileHandler.portInstances())

        if not success then
            warn('Argon: '..response..' (ui1)')
            FileHandler.clear()
        end

        if success then
            success, response = HttpHandler.portScripts(FileHandler.portScripts())

            if not success then
                warn('Argon: '..response..' (ui2)')
            end

            if success and Config.propertySyncing then
                success, response = HttpHandler.portProperties(FileHandler.portProperties())

                if not success then
                    warn('Argon: '..response..' (ui3)')
                end
            end
        end

        tween:Cancel()
        portToVSButton.Icon.Rotation = 0
        portToVSButton.Icon.Position = UDim2.fromScale(0.55, 0.5)
        portToVSButton.Icon.Image = START_ICON

        isPorting = false
    end
end

local function portToRoblox()
    if not isPorting and state == 1 then
        isPorting = true
        TwoWaySync.pause()

        local tween = TweenService:Create(portToRobloxButton.Icon, LOADING_TWEEN_INFO, {Rotation = -360})
        portToRobloxButton.Icon.Position = UDim2.fromScale(0.5, 0.5)
        portToRobloxButton.Icon.Image = LOADING_ICON
        tween:Play()

        local success, response = HttpHandler.portProject()

        if not success then
            warn('Argon: '..response..' (ui4)')
        end

        tween:Cancel()
        portToRobloxButton.Icon.Rotation = 0
        portToRobloxButton.Icon.Position = UDim2.fromScale(0.55, 0.5)
        portToRobloxButton.Icon.Image = START_ICON

        TwoWaySync.resume()
        isPorting = false
    end
end

local function updateTheme()
    local theme = Studio.Theme.Name

    if theme == lastTheme then
        return
    end
    lastTheme = theme

    if theme == 'Dark' then
        for _, v in ipairs(background:GetDescendants()) do
            if (v:IsA('Frame') and v.Name ~= 'Selector') or v:IsA('ImageButton') then
                v.BackgroundColor3 = WHITE
            elseif v:IsA('TextBox') or v:IsA('TextLabel') then
                v.TextColor3 = LIGHT_WHITE
                if v:IsA('TextBox') then
                    v.BackgroundColor3 = WHITE
                end
            elseif v:IsA('ImageLabel') and v.Name ~= 'ClassIcon' and v.Name ~= 'Logo' then
                v.ImageColor3 = LIGHT_WHITE
            elseif v:IsA('ScrollingFrame') then
                v.ScrollBarImageColor3 = WHITE
            end
        end

        background.BackgroundColor3 = ROBLOX_BLACK
        mainPage.BackgroundColor3 = ROBLOX_BLACK
        settingsPage.BackgroundColor3 = ROBLOX_BLACK
        toolsPage.BackgroundColor3 = ROBLOX_BLACK
        playtestFrame.BackgroundColor3 = LIGHT_BLACK
    elseif theme == 'Light' then
        for _, v in ipairs(background:GetDescendants()) do
            if (v:IsA('Frame') and v.Name ~= 'Selector') or v:IsA('ImageButton') then
                v.BackgroundColor3 = BLACK
            elseif v:IsA('TextBox') or v:IsA('TextLabel') then
                v.TextColor3 = LIGHT_BLACK
                if v:IsA('TextBox') then
                    v.BackgroundColor3 = BLACK
                end
            elseif v:IsA('ImageLabel') and v.Name ~= 'ClassIcon' and v.Name ~= 'Logo' then
                v.ImageColor3 = LIGHT_BLACK
            elseif v:IsA('ScrollingFrame') then
                v.ScrollBarImageColor3 = BLACK
            end
        end

        background.BackgroundColor3 = ROBLOX_WHITE
        mainPage.BackgroundColor3 = ROBLOX_WHITE
        settingsPage.BackgroundColor3 = ROBLOX_WHITE
        toolsPage.BackgroundColor3 = ROBLOX_WHITE
        playtestFrame.BackgroundColor3 = LIGHT_WHITE
    end
end

function guiHandler.updateButton(newButton)
    button = newButton

    if state == 1 then
        button.Icon = CONNECTED_ICON
    end
end

function guiHandler.runPage(page)
    for _, v in pairs(connections) do
        v:Disconnect()
    end
    connections = {}

    if page == mainPage then
        connections['connectButton'] = connectButton.MouseButton1Click:Connect(connect)

        connections['hostInput'] = hostInput:GetPropertyChangedSignal('Text'):Connect(function() filterInput(0) end)
        connections['portInput'] = portInput:GetPropertyChangedSignal('Text'):Connect(function() filterInput(1) end)
        connections['hostInput2'] = hostInput.FocusLost:Connect(function() setAddress(hostInput, true) end)
        connections['portInput2'] = portInput.FocusLost:Connect(function() setAddress(portInput) end)

        connections['settingsButton'] = settingsButton.MouseButton1Click:Connect(function() changePage(-1.05, settingsPage, toolsPage) end)
        connections['toolsButton'] = toolsButton.MouseButton1Click:Connect(function() changePage(1.05, toolsPage, settingsPage) end)

        settingsPage.Body.ScrollingEnabled = false
    elseif page == settingsPage then
        connections['settingsBack'] = settingsBack.MouseButton1Click:Connect(function() changePage(0) end)

        connections['autoRunButton'] = autoRunButton.MouseButton1Click:Connect(function() toggleSetting(0) end)
        connections['autoReconnectButton'] = autoReconnectButton.MouseButton1Click:Connect(function() toggleSetting(1) end)
        connections['onlyCodeButton'] = onlyCodeButton.MouseButton1Click:Connect(function() toggleSetting(2) end)
        connections['openInEditorButton'] = openInEditorButton.MouseButton1Click:Connect(function() toggleSetting(3) end)
        connections['twoWaySyncButton'] = twoWaySyncButton.MouseButton1Click:Connect(function() toggleSetting(4) end)
        connections['propertySyncingButton'] = propertySyncingButton.MouseButton1Click:Connect(function() toggleSetting(5) end)
        connections['syncDuplicatesButton'] = syncDuplicatesButton.MouseButton1Click:Connect(function() toggleSetting(6) end)
        connections['classFilteringButton'] = classFilteringButton.MouseButton1Click:Connect(function() expandSetting('ClassFiltering') end)
        connections['syncedDirectoriesButton'] = syncedDirectoriesButton.MouseButton1Click:Connect(function() expandSetting('SyncedDirectories') end)

        settingsPage.Body.CanvasPosition = Vector2.new(0, 0)
        settingsPage.Body.ScrollingEnabled = true
    elseif page == toolsPage then
        connections['toolsBack'] = toolsBack.MouseButton1Click:Connect(function() changePage(0) end)

        connections['portToVSButton'] = portToVSButton.MouseButton1Click:Connect(portToVS)
        connections['portToRobloxButton'] = portToRobloxButton.MouseButton1Click:Connect(portToRoblox)

        settingsPage.Body.ScrollingEnabled = false
    end
end

function guiHandler.run(newPlugin, newWidget, newButton)
    themeConnection = themeConnection or Studio.ThemeChanged:Connect(updateTheme)
    versionLabel.Text = Config.argonVersion
    updateTheme()

    if not RunService:IsEdit() then
        playtestFrame.Visible = true
        return
    end

    plugin = newPlugin
    widget = newWidget
    button = newButton

    local hostSetting = plugin:GetSetting('Host')
    local portSetting = plugin:GetSetting('Port')
    local autoRunSetting = plugin:GetSetting('AutoRun')
    local autoReconnectSetting = plugin:GetSetting('AutoReconnect')
    local onlyCodeSetting = plugin:GetSetting('OnlyCode')
    local openInEditorSetting = plugin:GetSetting('OpenInEditor')
    local twoWaySyncSetting = plugin:GetSetting('TwoWaySync')
    local propertySyncingSetting = plugin:GetSetting('PropertySyncing')
    local syncDuplicatesSetting = plugin:GetSetting('SyncDuplicates')
    local filteringMode = plugin:GetSetting('FilteringMode')
    local filteredClassesSetting = plugin:GetSetting('FilteredClasses')
    local syncedDirectoriesSetting = plugin:GetSetting('SyncedDirectories')

    if hostSetting ~= nil then
        hostInput.Text = hostSetting
        Config.host = hostSetting
    end

    if portSetting ~= nil then
        portInput.Text = portSetting
        Config.port = portSetting
    end

    if autoRunSetting ~= nil then
        Config.autoRun = autoRunSetting

        if not autoRunSetting then
            autoRunButton.OnIcon.ImageTransparency = 1
        end
    end

    if autoReconnectSetting ~= nil then
        Config.autoReconnect = autoReconnectSetting

        if not autoReconnectSetting then
            autoReconnectButton.OnIcon.ImageTransparency = 1
        end
    end

    if onlyCodeSetting ~= nil then
        Config.onlyCode = onlyCodeSetting

        if not onlyCodeSetting then
            onlyCodeButton.OnIcon.ImageTransparency = 1
        end
    end

    if openInEditorSetting ~= nil then
        Config.openInEditor = openInEditorSetting

        if openInEditorSetting then
            openInEditorButton.OnIcon.ImageTransparency = 0
        end
    end

    if twoWaySyncSetting ~= nil then
        Config.twoWaySync = twoWaySyncSetting

        if twoWaySyncSetting then
            twoWaySyncButton.OnIcon.ImageTransparency = 0
        end
    end

    if propertySyncingSetting ~= nil then
        Config.propertySyncing = propertySyncingSetting

        if propertySyncingSetting then
            propertySyncingButton.OnIcon.ImageTransparency = 0
        end
    end

    if syncDuplicatesSetting ~= nil then
        Config.syncDuplicates = syncDuplicatesSetting

        if syncDuplicatesSetting then
            syncDuplicatesButton.OnIcon.ImageTransparency = 0
        end
    end

    if filteringMode ~= nil then
        Config.filteringMode = filteringMode

        if filteringMode then
            classFilteringFrame.Mode.Selector.Position = UDim2.fromScale(0.5, 0)
        end
    end

    Config.filteredClasses = filteredClassesSetting or Config.filteredClasses
    if filteredClassesSetting then
        local text = ''

        for i, v in ipairs(Config.filteredClasses) do
            if i ~= 1 then
                text = text..', '..v
            else
                text = v
            end
        end

        classFilteringFrame.Input.Text = text
    end

    Config.syncedDirectories = syncedDirectoriesSetting or Config.syncedDirectories
    for i, v in pairs(Config.syncedDirectories) do
        local properties = StudioService:GetClassIcon(i)
        local icon = syncedDirectoriesFrame[i].ClassIcon

        for j, w in pairs(properties) do
            icon[j] = w
        end

        if v then
            syncedDirectoriesFrame[i].Button.OnIcon.ImageTransparency = 0
        end
    end

    changePage(0)

    if Config.autoRun and state == 0 then
        connect()
    end

    if Config.openInEditor then
        handleDocumentChange()
    end

    if twoWaySyncSetting then
        TwoWaySync.run()
    end

    if not didSetup then
        didSetup = true

        plugin.Unloading:Once(function()
            HttpHandler.disconnect()
        end)
    end

    local update = HttpHandler.checkForUpdates()

    if typeof(update) == 'string' then
        updateFrame.Container.Title.Text = 'Argon '..update
        updateFrame.Visible = true

        updateFrame.Container.Button.MouseButton1Click:Once(function()
            updateFrame.Visible = false
        end)
    end
end

function guiHandler.stop()
    if RunService:IsEdit() then
        for _, v in pairs(connections) do
            v:Disconnect()
        end
        connections = {}
    end

    themeConnection:Disconnect()
    themeConnection = nil
end

return guiHandler