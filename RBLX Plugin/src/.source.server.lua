local Toolbar = require(script.Toolbar)
local GuiHandler = require(script.GuiHandler)

local button = Toolbar('Argon', plugin)

local widgetInfo = DockWidgetPluginGuiInfo.new(Enum.InitialDockState.Float, false, false, 400, 250, 400, 250)
local widget = plugin:CreateDockWidgetPluginGui('Argon', widgetInfo)

local isOpen = false

widget.Name = 'Argon'
widget.Title = 'Argon'
widget.ZIndexBehavior = Enum.ZIndexBehavior.Sibling
script.Parent.ArgonGui.Root.Background.Parent = widget

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

button.Click:Connect(function()
    if isOpen then
        close()
    else
        open()
    end
end)

widget.WindowFocused:Connect(open)
widget:BindToClose(close)

if widget.Enabled then
    open()
elseif plugin:GetSetting('AutoRun') then
    GuiHandler.run(plugin, widget, button)
end