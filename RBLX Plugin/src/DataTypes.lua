local HttpService = game:GetService('HttpService')

local Config = require(script.Parent.Config)

local URL = 'http://%s:%s/'
local ARGON_UUID = 'ArgonUUID'
local DATA_TYPES = {
    ['Vector3'] = Vector3,
    ['Vector2'] = Vector2,
    ['CFrame'] = CFrame,
    ['Color3'] = Color3,
    ['UDim2'] = UDim2,
    ['UDim'] = UDim,
    ['Rect'] = Rect,
    ['NumberRange'] = NumberRange,
    ['PhysicalProperties'] = PhysicalProperties
}

local apiDump = nil

local dataTypes = {}

local function getApiDump()
    local url = URL:format(Config.host, Config.port)
    local header = {action = 'getApiDump'}

    local success, response = pcall(function()
        apiDump = HttpService:JSONDecode(HttpService:GetAsync(url, false, header))
    end)

    if not success then
        warn('Argon: '..response..' (dt)')
    end

    return success
end

local function getPath(instance)
    local path = ''

    if instance.Parent ~= game then
        path = getPath(instance.Parent)..Config.separator..instance.Name
    else
        path = instance.ClassName
    end

    return path
end

local function getInstance(parent)
    parent = parent:split(Config.separator)
    local lastParent = game

    for _, v in ipairs(parent) do
        if lastParent == game then
            lastParent = game:GetService(v)
        else
            local didFind = false
            local uuid = nil

            if Config.syncDuplicates and v:find('%%') and v:len() - v:find('%%') == 6 then
                local temp = v
                v = temp:sub(1, temp:len() - 7)
                uuid = temp:sub(temp:len() - 5)
            end

            for _, w in ipairs(lastParent:GetChildren()) do
                if not uuid then
                    if w.Name == v then
                        lastParent = w
                        didFind = true
                        break
                    end
                else
                    if w.Name == v and w:GetAttribute(ARGON_UUID) == uuid then
                        lastParent = w
                        didFind = true
                        break
                    end
                end
            end

            if not didFind then
                return
            end
        end
    end

    return lastParent
end

local function stringify(value)
    local dataType = typeof(value)

    if dataType == 'boolean' or dataType == 'number' or dataType == 'string' then
        return value

    elseif DATA_TYPES[dataType] then
        if dataType == 'CFrame' then
            return {value:GetComponents()}
        elseif dataType == 'Vector3' then
            return {value.X, value.Y, value.Z}
        elseif dataType == 'Vector2' then
            return {value.X, value.Y}
        elseif dataType == 'Color3' then
            return {value.R, value.G, value.B}
        elseif dataType == 'UDim2' then
            return {value.X.Scale, value.X.Offset, value.Y.Scale, value.Y.Offset}
        elseif dataType == 'UDim' then
            return {value.Scale, value.Offset}
        elseif dataType == 'Rect' then
            return {value.Min.X, value.Min.Y, value.Max.X, value.Max.Y}
        elseif dataType == 'NumberRange' then
            return {value.Min, value.Max}
        elseif dataType == 'PhysicalProperties' then
            return {value.Density, value.Friction, value.Elasticity, value.FrictionWeight, value.ElasticityWeight}
        end

    elseif dataType == 'EnumItem' or dataType == 'BrickColor' then
        return tostring(value)

    elseif dataType == 'Font' then
        return {value.Family, tostring(value.Weight), tostring(value.Style)}

    elseif dataType == 'Faces' or dataType == 'Axes' then
        return tostring(value):split(', ')

    elseif dataType == 'NumberSequence' then
        local newValue = {}

        for _, w in ipairs(value.Keypoints) do
            table.insert(newValue, {w.Time, w.Value, w.Envelope})
        end

        return newValue

    elseif dataType == 'ColorSequence' then
        local newValue = {}

        for _, w in ipairs(value.Keypoints) do
            table.insert(newValue, {w.Time, w.Value.R, w.Value.G, w.Value.B})
        end

        return newValue

    elseif dataType == 'Instance' then
        return getPath(value)
    end
end

function dataTypes.cast(value, property, object)
    if typeof(value) == 'boolean' or typeof(value) == 'number' then
        return value
    end

    local dataType

    if object then
        dataType = typeof(object[property])
    else
        dataType = property
    end

    if dataType == 'string' then
        return value

    elseif DATA_TYPES[dataType] then
        return DATA_TYPES[dataType].new(unpack(value))

    elseif dataType == 'EnumItem' then
        value = value:split('.')
        return Enum[value[2]][value[3]]

    elseif dataType == 'BrickColor' then
        return BrickColor.new(value)

    elseif dataType == 'Font' then
        return Font.new(value[1], Enum.FontWeight[value[2]:split('.')[3]], Enum.FontStyle[value[3]:split('.')[3]])

    elseif dataType == 'Faces' then
        for i, v in ipairs(value) do
            value[i] = Enum.NormalId[v]
        end

        return Faces.new(unpack(value))

    elseif dataType == 'Axes' then
        for i, v in ipairs(value) do
            value[i] = Enum.Axis[v]
        end

        return Axes.new(unpack(value))

    elseif dataType == 'NumberSequence' then
        local keypoints = {}

        for _, v in ipairs(value) do
            table.insert(keypoints, NumberSequenceKeypoint.new(v[1], v[2], v[3]))
        end

        return NumberSequence.new(keypoints)

    elseif dataType == 'ColorSequence' then
        local keypoints = {}

        for _, v in ipairs(value) do
            table.insert(keypoints, ColorSequenceKeypoint.new(v[1], Color3.new(v[2], v[3], v[4])))
        end

        return ColorSequence.new(keypoints)

    else
        return getInstance(value)
    end
end

function dataTypes.getProperties(object)
    if not apiDump then
        if not getApiDump() then
            return
        end
    end

    if not apiDump[object.ClassName] then
        return
    end

    local properties = {}

    properties.Class = object.ClassName

    for _, v in ipairs(apiDump[object.ClassName]) do
        local value = object[v]

        if not value then
            continue
        end

        properties[v] = stringify(value)
    end

    for i, v in pairs(object:GetAttributes()) do
        if not properties.Attributes then
            properties.Attributes = {}
        end

        properties.Attributes[i] = {Type = typeof(v), Value = stringify(v)}
    end

    return properties
end

return dataTypes