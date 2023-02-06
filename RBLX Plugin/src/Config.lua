local config = {}

config.argonVersion = '1.1.0'

config.host = 'localhost'
config.port = '8000'

config.autoRun = true
config.autoReconnect = true
config.openInEditor = true
config.onlyCode = true
config.twoWaySync = false
config.propertySyncing = false

config.filteringMode = false
config.filteredClasses = {}

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

return config