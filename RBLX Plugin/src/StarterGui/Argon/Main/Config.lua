local config = {}

config.argonVersion = '0.5.1'

config.autoRun = true
config.autoReconnect = false
config.openInEditor = false
config.onlyCode = true
config.twoWaySync = false
config.propertySyncing = false

config.host = 'localhost'
config.port = '8000'

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