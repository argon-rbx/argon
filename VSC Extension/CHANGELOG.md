# Changelog

# 1.2.1
* Added automatic Git repo initialization (disabled by default), suggested by [@AridTheDev](https://devforum.roblox.com/u/aridthedev)
* Fixed issue with porting to VSC caused by UUIDs, reported by [@rick2809](https://devforum.roblox.com/u/rick2809)
* Open in editor option no longer works when two-way sync is enabled

# 1.2.0
* Added full support for macOS
* Added support for multiple instances with the same name inside the same directory
* Fixed custom dirs behavior when using root path
* Fixed instance attributes not being removed
* Fixed and improved two-way sync

# 1.1.0
* Added support for Wally
* Added support for instance attributes, suggested by [@AridTheDev](https://devforum.roblox.com/u/aridthedev)
* Added support for my other plugins (shared toolbar)
* Fixed plugin not detecting first input
* Fixed custom font not loading on local website
* Fixed status bar icon not showing sometimes
* Optimized porting related code
* Improved stats counting
* Renamed some commands

## 1.0.2
* Unhandled properties in the project file no longer cause problems, reported by [@AridTheDev](https://devforum.roblox.com/u/aridthedev)
* Fixed intellisense not working for newly created scripts, reported by [@AridTheDev](https://devforum.roblox.com/u/aridthedev)
* Fixed custom paths working improperly, reported by [@dz_Scy](https://devforum.roblox.com/u/dz_scy)
* Updated wiki

## 1.0.1
* Added version mismatch detection
* Fixed project name syncing
* Fixed "hours used" global statistic

## 1.0.0
* Argon is no longer in beta
* Added global stats badges and [website](https://argonstatsapi.web.app/)
* Added "Remove Studio Shortcut" setting
* Added support for custom service directories, suggested by [@dz_Scy](https://devforum.roblox.com/u/dz_scy)
* Added support for "Start local server" option in playtest commands
* Added back support for custom instances in "StarterPlayer"
* Plugin title now uses VSC workspace name by default and updates in real time
* VSC status bar icon now displays current server address
* Improved Roblox plugin UI scaling
* Improved websocket managing
* Updated local website
* Updated wiki

## 0.6.5
* Finished Argon [wiki](https://github.com/DervexHero/Argon/wiki)
* Updated plugin window behavior when playtesting and closing
* Updated markdowns

## 0.6.4
* Added connection status to Roblox plugin icon, suggested by [@ecndm70](https://devforum.roblox.com/u/ecndm70)
* Added option to automatically switch to Studio when using execute snippet command
* Added Argon [wiki](https://github.com/DervexHero/Argon/wiki) on GitHub (work in progress)
* Improved script with children detection, thanks to [@AridTheDev](https://devforum.roblox.com/u/aridthedev)
* Improved status bar icon

## 0.6.3
* Added execute snippet command (quick pick menu and F6)
* Added status bar icon, suggested by [@ecndm70](https://devforum.roblox.com/u/ecndm70)
* Argon now uses HTML file as a local website
* Fixed some networking issues

## 0.6.2
* Added support for name used in default.project.json (only if ~= "Argon"), suggested by [@AridTheDev](https://devforum.roblox.com/u/aridthedev)
* Fixed bug reported by [@AridTheDev](https://devforum.roblox.com/u/aridthedev) that crashed VSC extension when trying to port empty folders
* Fixed bug reported by [@ecndm70](https://devforum.roblox.com/u/ecndm70) was causing errors with BindToClose event
* Fixed JSON schema not working in default.project.json

## 0.6.1
* Added support for instance references (property type)
* Added dynamic plugin and extension title (displays current project), suggested by [@Plasmanode](https://devforum.roblox.com/u/plasma_node)
* Added new debugging menu and removed debugging mode setting
* You can now play or run playtest with different keybinds (F5 and F8) in VSC
* Fixed BrickColor property not porting from VSC to Studio

## 0.6.0
* Added support for external tooling
* Roblox LSP is now natively supported
* Clicking on output prints/errors will now get you to the right line in VSC, only when open in editor is enabled (requested by [@Plasmanode](https://devforum.roblox.com/u/plasma_node))
* Added compatibility mode which replaces ".source" files with "init" and .properties files with "init.meta"
* Instance classes are now stored in .properties.json only when property syncing option is enabled
* Replaced .argon.json with default.project.json (required to for Roblox LSP and other tooling)
* Removed update classes command and auto update option - no real use cases (still available in [jsonGenerator.js](https://github.com/DervexHero/Argon/blob/main/VSC%20Extension/src/utils/jsonGenerator.js))
* Auto Studio launch now checks if Roblox is already running
* Argon now utilizes new Roblox [ScriptEditorService](https://create.roblox.com/docs/reference/engine/classes/ScriptEditorService) API
* Modified existing extension options (auto setup)
* Tons of bug fixes and code optimizations
* Redesigned quick pick menu

## 0.5.3
* Fixed bug reported by [@Loomiquu](https://github.com/Loomiquu) in [#6](https://github.com/DervexHero/Argon/issues/6)

## 0.5.2
* Fixed bug reported by [@0MRob](https://devforum.roblox.com/u/0mrob) that blocked script source from being ported when script had non-script children
* Fixed properties not porting when instance has no children
* Fixed ArgonIgnore attribute not working in only code mode
* Fixed UDim and UDim2 properties not porting
* Fixed Argon logo on local website
* Now porting is even faster

## 0.5.1
* Fixed StarterPlayer services not porting
* Fixed open in editor option not working properly
* Fixed .properties file behavior when porting to Roblox
* Fixed properties of main services e.g. Workspace not syncing when only code mode was disabled
* Fixed --disable flag inside scripts not working when porting to VSC
* Argon now ignores and does not create empty .properties files
* Lua files inside root folder no longer yield whole code
* Local API dump no longer contains empty arrays
* Removed memory leak from winuser.cpp
* Other small fixes

## 0.5.0
* Added property sync
* Added start debugging option
* Added auto open Roblox option
* Added JSON schema for .properties file
* Extension no longer uses node-ffi-napi library (used native C++ instead)
* Changed StarterPlayer services name inside VSC
* "directory" is not longer required in .argon.json
* Fixed open in editor option not working after playtest
* Fixed destroying non script instances not working

## 0.4.3
* Added new Argon menu (old commands won't work anymore!)
* Added [JSON Schema](https://github.com/DervexHero/Argon/blob/main/VSC%20Extension/config/.argon.schema.json) file to make editing ".argon.json" easier
* Fixed instances with special characters in their names not syncing from VSC to Roblox
* Fixed Roblox Studio updating every time you use launch Roblox Studio command
* Fixed open in editor option not working after leaving settings
* Fixed version detection API (plugin and extension were getting wrong values)
* Upgraded open in editor option - now scripts inside Studio will close automatically

## 0.4.2
* Fixed critical VSC extension bug (excluded missing dependencies from .vscodeignore)

## 0.4.1
* Temporary hotfix for VSC extension (extension could not run)

## 0.4.0
* Added two-way sync (only for code)
* Added "Launch Roblox Studio" command (VSC)
* Added "Open In Editor" option (Roblox)
* Added open in preview option (VSC, settings)
* Fixed code not syncing when only code mode was enabled and script instance had children
* VSC extension now utilizes [node-ffi-napi](https://github.com/node-ffi-napi/node-ffi-napi) to communicate with user32.dll (required to bypass Windows limitation and force editor to open)
* Many code optimizations

## 0.3.4
* Fixed bug reported by [@Plasmanode](https://devforum.roblox.com/u/plasma_node) that caused errors when porting "StarterPlayer" to VSC
* Fixed bug reported by [@Plasmanode](https://devforum.roblox.com/u/plasma_node) that caused descendants of the scripts being overwritten
* Fixed source code being ported twice (from VSC to Roblox)
* Fixed script source not porting in some cases
* Porting source code takes less time
* All plugin icons are now preloaded
* Minor UI changes

## 0.3.3
* Added icon for .argon.json
* Added support for custom directories (inside .argon.json)
* Fixed issue with root folder caused by 0.3.2 update
* Fixed "cannot resume dead coroutine" error

## 0.3.2
* Added auto check for updates in both Roblox plugin and VSC extension
* Added option to not create root (src by default) folder automatically (VSC)
* Fixed some bugs which were caused by the lack of an open workspace (VSC)
* Updating Argon settings no longer requires window reload (VSC)
* Added more VSC icons

## 0.3.1
* Removed node_modules from VSC extension

## 0.3.0
* Added releases on Github
* Added local website with Argon stats
* Added option to sync only code (exclude empty instances)
* Added option for VSC extension to suppress notifications
* Connecting multiple clients to one Argon server is no longer possible
* StarterPlayer no longer ports to VSC if StarterPlayerScripts and StarterCharacterScripts are empty
* Fixed extension not stopping when plugin was still sending requests (temp fix by destroying websockets, this method will change when Electron adds support for node.js 18.2.0+)
* Fixed notifications displaying twice when Argon got enabled with command
* Fixed class filtering selector color on light mode
* Fixed plugin auto run option
* Various code optimizations

## 0.2.1 - 0.2.3
* Repository and marketplace modifications

## 0.2.0
* Added whitelist system for class filtering, suggested by [@Punctuation](https://devforum.roblox.com/u/loomiquu)
* Fixed unclickable buttons after playtest, reported by [@AridTheDev](https://devforum.roblox.com/u/aridthedev)
* Fixed default values of VSC extension settings
* Fixed light theme and UI scaling
* Tons of other UI optimizations and fixes
* Updated some UI buttons
* Updated VSC icons

## 0.1.4
* Instances named same as parent's property no longer cause problems
* Fixed some error messages
* Added icon for .vsix files

## 0.1.3
* Now Argon will automatically run once you open .lua or .luau file
* Added more detailed error messages

## 0.1.2
* Replaced heavy GIFs with videos
* Cleaned .ignore files

## 0.1.1
* Added better documentation
* Added new icons
* Added invalid symbols detection
* Fixed ignored classes
* Fixed .source deletion behavior (recursive rmSync)
* Made StarterPlayerScripts and StarterCharacterScripts detection server sided

## 0.1.0
* Added port to Roblox feature
* Changed default root folder name to "src" for convenience
* Fixed duplicated instances bug

## 0.0.9
* Merged changes by [@Almost89](https://github.com/Almost89) in [#1](https://github.com/DervexHero/Argon/pull/1)
* Fixed issue reported by [@LawMixer](https://devforum.roblox.com/u/bulldo344), [@commitblue](https://devforum.roblox.com/u/commitblue), [@Voidage](https://devforum.roblox.com/u/voidage) which made users unable to port large places
* Added support for renamed services
* Added history service support
* Added error ids
* Added more icons
* Moved plugin source to Argon

## 0.0.8
* Fixed bug that caused not changing .source file type

## 0.0.7
* Added Argon file icon theme

## 0.0.6
* Added support for deleting, creating and moving multiple files at once
* Moving service folders e.g. Workspace no longer causes errors
* Creating .source file inside root folder no long causes errors
* Moving .source file is no longer allowed

## 0.0.5
* Fixed critical issue
* Added changelog
## 0.0.4
* Finished porting feature
* Added VS Code extension options

## 0.0.3
* Added port to VS Code feature (not completed)

## 0.0.2
* Updated Roblox plugin UI
* Added markdown files

## 0.0.1
* Initial release