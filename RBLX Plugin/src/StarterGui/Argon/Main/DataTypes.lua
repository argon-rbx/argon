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

local dataTypes = {}

--TODO: font, axes, faces, number range

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

return dataTypes