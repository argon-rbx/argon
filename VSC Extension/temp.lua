local test = 'Enum.EasingDirection.InOut'

local function load(src)
    return loadstring('return '..src)()
end

print(load(test))