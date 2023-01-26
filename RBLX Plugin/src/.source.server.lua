local Toolbar = require(script.Toolbar)
local GuiHandler = require(script.GuiHandler)

local widgetInfo = DockWidgetPluginGuiInfo.new(Enum.InitialDockState.Float, false, false, 400, 250, 400, 250)
local widget = plugin:CreateDockWidgetPluginGui('Argon', widgetInfo)

local isOpen = false
local connection = nil

widget.Name = 'Argon'
widget.Title = 'Argon'
widget.ZIndexBehavior = Enum.ZIndexBehavior.Sibling
script.Parent.ArgonGui.Root.Background.Parent = widget

button = Toolbar(plugin, 'Argon', 'rbxassetid://11230142853', function(newButton)
    GuiHandler.updateButton(newButton)
    button = newButton

    if isOpen then
        button:SetActive(true)
    end

    if connection then
        connection:Disconnect()
        connection = nil
    end

    connection = button.Click:Connect(openClose)
end)

local function open()
    if not isOpen then
        isOpen = true
        button:SetActive(true)
        GuiHandler.run(plugin, widget, button)
        widget.Enabled = true
    end
end

local function close()
    if isOpen then
        isOpen = false
        button:SetActive(false)
        GuiHandler.stop()
        widget.Enabled = false
    end
end

function openClose()
    if isOpen then
        close()
    else
        open()
    end
end

connection = button.Click:Connect(openClose)
widget.WindowFocused:Connect(open)
widget:BindToClose(close)

if widget.Enabled then
    open()
elseif plugin:GetSetting('AutoRun') then
    GuiHandler.run(plugin, widget, button)
end