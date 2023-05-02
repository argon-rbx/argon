local CoreGui = game:GetService('CoreGui')

local TOOLBAR = "Dervex' utils"

return function (plugin: Plugin, name: string, icon: string, priority: number, callback: (PluginToolbarButton) -> ())
    task.wait(priority / 10)

    local toolbarRef, buttonRef
    local toolbar, button
    local isOwner = false

    local function createButton(init)
        if toolbarRef:FindFirstChild(name) then
            buttonRef = toolbarRef[name]
            button = buttonRef.Value
            button.Enabled = true
        else
            button = toolbar:CreateButton(name, 'Show '..name..' UI', icon)
            button.ClickableWhenViewportHidden = true

            buttonRef = Instance.new('ObjectValue', toolbarRef)
            buttonRef.Name = name
            buttonRef.Value = button

            if callback and not init then
                callback(button)
            end
        end
    end

    local function createToolbar(init)
        if CoreGui:FindFirstChild(TOOLBAR) then
            toolbarRef = CoreGui[TOOLBAR]
            toolbar = toolbarRef.Value

            toolbarRef.Destroying:Once(function()
                task.delay(0, createToolbar)
            end)

            createButton(init)
        else
            toolbar = plugin:CreateToolbar(TOOLBAR)
            isOwner = true

            toolbarRef = Instance.new('ObjectValue', CoreGui)
            toolbarRef.Name = TOOLBAR
            toolbarRef.Value = toolbar

            createButton(init)
        end
    end

    createToolbar(true)

    plugin.Unloading:Once(function()
        button.Enabled = false

        if isOwner then
            toolbarRef:Destroy()
        end
    end)

    return button
end