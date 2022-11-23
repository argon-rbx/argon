local config = {}

config.argonVersion = '0.3.0'

config.autoRun = true
config.autoReconnect = false
config.twoWaySync = false
config.onlyCode = true

config.host = 'localhost'
config.port = '8000'

config.syncedDirectories = {
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

config.filteringMode = false
config.filteredClasses = {}

return config