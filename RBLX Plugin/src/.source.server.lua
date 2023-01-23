local CoreGui = game:GetService('CoreGui')

local GuiHandler = require(script.GuiHandler)

local toolbar

if CoreGui:FindFirstChild("Dervex' utils") and #CoreGui["Dervex' utils"]:GetChildren() > 0 then
    toolbar = CoreGui["Dervex' utils"].Value
else
    if CoreGui:FindFirstChild("Dervex' utils") then
        CoreGui["Dervex' utils"]:Destroy()
    end

    toolbar = plugin:CreateToolbar("Dervex' utils")

    local toolbarRef = Instance.new('ObjectValue', CoreGui)
    toolbarRef.Name = "Dervex' utils"
    toolbarRef.Value = toolbar
end

local button = toolbar:CreateButton('Argon', 'Show Argon UI', 'rbxassetid://11230142853')

local widgetInfo = DockWidgetPluginGuiInfo.new(Enum.InitialDockState.Float, false, false, 400, 250, 400, 250)
local widget = plugin:CreateDockWidgetPluginGui('Argon', widgetInfo)

local isOpen = false

widget.Name = 'Argon'
widget.Title = 'Argon'
widget.ZIndexBehavior = Enum.ZIndexBehavior.Sibling
script.Parent.ArgonGui.Root.Background.Parent = widget
button.ClickableWhenViewportHidden = true
Instance.new('ObjectValue', CoreGui["Dervex' utils"]).Name = 'Argon'

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

plugin.Unloading:Connect(function()
    if CoreGui:FindFirstChild("Dervex' utils") and CoreGui["Dervex' utils"]:FindFirstChild('Argon') then
        CoreGui["Dervex' utils"].Argon:Destroy()
    end
end)