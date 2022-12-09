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
    local url = string.format(URL, Config.host, Config.port)
    local header = {action = 'getApiDump'}

    local success, response = pcall(function()
        apiDump = HttpService:JSONDecode(HttpService:GetAsync(url, false, header))
    end)

    if not success then
        warn('Argon: '..response..' (dt)')
    end

    return success
end

--TODO: font, axes, faces, number seq

function dataTypes.cast(value, property, object)
    if typeof(value) == 'boolean' or typeof(value) == 'number' then
        return value
    end

    local dataType = typeof(object[property])

    if dataType == 'string' then
        return value
    elseif DATA_TYPES[dataType] then
        return DATA_TYPES[dataType].new(unpack(value))
    elseif dataType == 'BrickColor' then
        return BrickColor.new(value)
    elseif dataType == 'EnumItem' then
        value = value:split('.')
        return Enum[value[2]][value[3]]
    end
end

function dataTypes.getProperties(object)
    if not apiDump then
        local success = getApiDump()

        if not success then
            return
        end
    end

    local properties = {}

    for _, v in ipairs(apiDump[object.ClassName]) do
        local dataType = typeof(object[v])

        if dataType == 'boolean' or dataType == 'number' or dataType == 'string' then
            properties[v] = object[v]
        elseif DATA_TYPES[dataType] then
            if dataType == 'CFrame' then
                properties[v] = {object[v]:GetComponents()}
            elseif dataType == 'Vector3' then
                properties[v] = {object[v].X, object[v].Y, object[v].Z}
            elseif dataType == 'Vector2' then
                properties[v] = {object[v].X, object[v].Y}
            elseif dataType == 'Color3' then
                properties[v] = {object[v].R, object[v].G, object[v].B}
            elseif dataType == 'Udim2' then
                properties[v] = {object[v].X.Scale, object[v].X.Offset, object[v].Y.Scale, object[v].Y.Offset}
            elseif dataType == 'Udim' then
                properties[v] = {object[v].Scale, object[v].Offset}
            elseif dataType == 'Rect' then
                properties[v] = {object[v].Min.X, object[v].Min.Y, object[v].Max.X, object[v].Max.Y}
            elseif dataType == 'NumberRange' then
                properties[v] = {object[v].Min, object[v].Max}
            elseif dataType == 'PhysicalProperties' then
                properties[v] = {object[v].Density, object[v].Friction, object[v].Elasticity, object[v].FrictionWeight, object[v].ElasticityWeight}
            end
        else
            properties[v] = tostring(object[v])
        end
    end

    return properties
end

return dataTypes