local dataTypes = {}

function dataTypes.cast(property)
    if typeof(property) == 'boolean' or typeof(property) == 'number' then
        return property
    end

    if string.find(property, '.') then
        if string.match(property, '^Enum') then
            property = string.split(property, '.')
            return Enum[property[2]][property[3]]
        --elseif  then
        end
    else
        return property
    end
end

return dataTypes