local CoreGui = game:GetService('CoreGui')

local TOOLBAR = "Dervex' utils"

return function (name, plugin)
    local toolbar, button
    local toolbarRef, buttonRef

    if CoreGui:FindFirstChild(TOOLBAR) and #CoreGui[TOOLBAR]:GetChildren() > 0 then
        toolbarRef = CoreGui[TOOLBAR]
        toolbar = toolbarRef.Value
    else
        if CoreGui:FindFirstChild(TOOLBAR) then
            CoreGui[TOOLBAR]:Destroy()
        end

        toolbar = plugin:CreateToolbar(TOOLBAR)

        toolbarRef = Instance.new('ObjectValue', CoreGui)
        toolbarRef.Name = TOOLBAR
        toolbarRef.Value = toolbar
    end

    if toolbarRef:FindFirstChild(name) then
        buttonRef = toolbarRef[name]
        button = buttonRef.Value
        button.Enabled = true
    else
        button = toolbar:CreateButton(name, 'Show '..name..' UI', 'rbxassetid://12243614008')
        button.ClickableWhenViewportHidden = true

        buttonRef = Instance.new('ObjectValue', toolbarRef)
        buttonRef.Name = name
        buttonRef.Value = button
    end

    plugin.Unloading:Once(function()
        button.Enabled = false
    end)

    return button
end