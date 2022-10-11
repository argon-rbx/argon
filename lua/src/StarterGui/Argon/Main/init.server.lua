local GuiHandler = require(script.GuiHandler)

local toolbar = plugin:CreateToolbar("Dervex' utils")
local button = toolbar:CreateButton('Argon', 'Show Argon UI', 'rbxassetid://11230142853')

local widgetInfo = DockWidgetPluginGuiInfo.new(Enum.InitialDockState.Float, false, false, 500, 300, 250, 150)
local widget = plugin:CreateDockWidgetPluginGui('Argon', widgetInfo)

local isOpen = false

widget.Title = 'Argon'
widget.ZIndexBehavior = Enum.ZIndexBehavior.Sibling
script.Parent.ArgonGui.Root.Background.Parent = widget
button.ClickableWhenViewportHidden = true

local function open()
    if not isOpen then
        isOpen = true
        button:SetActive(true)
        GuiHandler.init(plugin)
        GuiHandler.run()
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

widget:BindToClose(function()
    close()
end)