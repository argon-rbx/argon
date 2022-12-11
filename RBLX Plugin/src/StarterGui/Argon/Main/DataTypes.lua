local HttpService = game:GetService('HttpService')

local Config = require(script.Parent.Config)

local URL = 'http://%s:%s/'
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

function dataTypes.cast(value, property, object)
    if typeof(value) == 'boolean' or typeof(value) == 'number' then
        return value
    end

    local dataType = typeof(object[property])

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
    end
end

function dataTypes.getProperties(object)
    if not apiDump then
        if not getApiDump() then
            return
        end
    end

    if not apiDump[object.ClassName] then
        return nil
    end

    local properties = {}

    for _, v in ipairs(apiDump[object.ClassName]) do
        local dataType = typeof(object[v])
        local value = object[v]

        if dataType == 'boolean' or dataType == 'number' or dataType == 'string' then
            properties[v] = value

        elseif DATA_TYPES[dataType] then
            if dataType == 'CFrame' then
                properties[v] = {value:GetComponents()}
            elseif dataType == 'Vector3' then
                properties[v] = {value.X, value.Y, value.Z}
            elseif dataType == 'Vector2' then
                properties[v] = {value.X, value.Y}
            elseif dataType == 'Color3' then
                properties[v] = {value.R, value.G, value.B}
            elseif dataType == 'Udim2' then
                properties[v] = {value.X.Scale, value.X.Offset, value.Y.Scale, value.Y.Offset}
            elseif dataType == 'Udim' then
                properties[v] = {value.Scale, value.Offset}
            elseif dataType == 'Rect' then
                properties[v] = {value.Min.X, value.Min.Y, value.Max.X, value.Max.Y}
            elseif dataType == 'NumberRange' then
                properties[v] = {value.Min, value.Max}
            elseif dataType == 'PhysicalProperties' then
                properties[v] = {value.Density, value.Friction, value.Elasticity, value.FrictionWeight, value.ElasticityWeight}
            end

        elseif dataType == 'EnumItem' then
            properties[v] = tostring(value)

        elseif dataType == 'Font' then
            properties[v] = {value.Family, tostring(value.Weight), tostring(value.Style)}

        elseif dataType == 'Faces' or dataType == 'Axes' then
            properties[v] = tostring(value):split(', ')

        elseif dataType == 'NumberSequence' then
            local newValue = {}
            for _, w in ipairs(value.Keypoints) do
                table.insert(newValue, {w.Time, w.Value, w.Envelope})
            end
            properties[v] = newValue

        elseif dataType == 'ColorSequence' then
            local newValue = {}
            for _, w in ipairs(value.Keypoints) do
                table.insert(newValue, {w.Time, w.Value.R, w.Value.G, w.Value.B})
            end
            properties[v] = newValue
        end
    end

    return properties
end

return dataTypes