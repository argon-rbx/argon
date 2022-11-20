local data = {}

data.argonVersion = '0.2.0'

data.autoRun = true
data.autoReconnect = false

data.host = 'localhost'
data.port = '8000'

data.syncedDirectories = {
    ['Workspace'] = false,
    ['Players'] = false,
    ['Lighting'] = false,
    ['MaterialService'] = false,
    ['ReplicatedFirst'] = true,
    ['ReplicatedStorage'] = true,
    ['ServerScriptService'] = true,
    ['ServerStorage'] = true,
    ['StarterGui'] = true,
    ['StarterPack'] = true,
    ['StarterPlayer'] = true,
    ['Teams'] = false,
    ['SoundService'] = false,
    ['Chat'] = false,
    ['LocalizationService'] = false,
    ['TestService'] = false
}

data.filteringMode = false
data.filteredClasses = {}

return data