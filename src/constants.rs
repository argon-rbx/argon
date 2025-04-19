use std::{sync::OnceLock, time::Duration};

use crate::{core::meta::SyncRule, middleware::Middleware};

// Paths that should be ignored before they are even processed
// useful to save ton of computing time, however users won't
// be able to set them in `sync_rules` or project `$path`
pub const BLACKLISTED_PATHS: [&str; 1] = [".DS_Store"];

// Current version of the project templates, this constant
// should be manually bumped when there are any changes
// made to the `assets/templates` directory
pub const TEMPLATES_VERSION: u8 = 4;

// Maximum payload size that can be sent from client
// to the server, usually containing changes to apply,
// currently it is 512 MiB but it is a huge overkill
pub const MAX_PAYLOAD_SIZE: usize = 536_870_912;

/// How long the server should wait for the changes to
/// appear in the queue before manually "timing out"
/// the client request and sending back an empty `Changes`
pub const QUEUE_TIMEOUT: Duration = Duration::from_secs(60);

// VFS events will be ignored for this amount of time
// after the last change that has been made by the client,
// this saves a lot of computing time
pub const SYNCBACK_DEBOUNCE_TIME: Duration = Duration::from_millis(200);

// Set of default sync rules that is used to determine
// what middleware should be used to process a file
// users can override these rules in the project file
pub fn default_sync_rules() -> &'static Vec<SyncRule> {
	static SYNC_RULES: OnceLock<Vec<SyncRule>> = OnceLock::new();

	SYNC_RULES.get_or_init(|| {
		vec![
			// Project and data files
			SyncRule::new(Middleware::Project)
				.with_pattern("*.project.json")
				.with_child_pattern("default.project.json"),
			SyncRule::new(Middleware::InstanceData)
				.with_pattern("*.data.json")
				.with_child_pattern(".data.json"),
			SyncRule::new(Middleware::InstanceData)
				.with_pattern("*.meta.json")
				.with_child_pattern("init.meta.json"),
			//////////////////////////////////////////////////////////////////////////////////////////
			// Luau scripts
			SyncRule::new(Middleware::ServerScript)
				.with_pattern("*.server.luau")
				.with_child_pattern("init.server.luau")
				.with_suffix(".server.luau"),
			SyncRule::new(Middleware::ClientScript)
				.with_pattern("*.client.luau")
				.with_child_pattern("init.client.luau")
				.with_suffix(".client.luau"),
			SyncRule::new(Middleware::ModuleScript)
				.with_pattern("*.luau")
				.with_child_pattern("init.luau"),
			// Luau scripts for Argon Legacy
			SyncRule::new(Middleware::ServerScript)
				.with_pattern("*.server.luau")
				.with_child_pattern(".src.server.luau")
				.with_suffix(".server.luau")
				.with_exclude("init.server.luau"),
			SyncRule::new(Middleware::ClientScript)
				.with_pattern("*.client.luau")
				.with_child_pattern(".src.client.luau")
				.with_suffix(".client.luau")
				.with_exclude("init.client.luau"),
			SyncRule::new(Middleware::ModuleScript)
				.with_pattern("*.luau")
				.with_child_pattern(".src.luau")
				.with_exclude("init.luau"),
			//////////////////////////////////////////////////////////////////////////////////////////
			// Lua scripts
			SyncRule::new(Middleware::ServerScript)
				.with_pattern("*.server.lua")
				.with_child_pattern("init.server.lua")
				.with_suffix(".server.lua"),
			SyncRule::new(Middleware::ClientScript)
				.with_pattern("*.client.lua")
				.with_child_pattern("init.client.lua")
				.with_suffix(".client.lua"),
			SyncRule::new(Middleware::ModuleScript)
				.with_pattern("*.lua")
				.with_child_pattern("init.lua"),
			// Lua scripts for Argon legacy
			SyncRule::new(Middleware::ServerScript)
				.with_pattern("*.server.lua")
				.with_child_pattern(".src.server.lua")
				.with_suffix(".server.lua")
				.with_exclude("init.server.lua"),
			SyncRule::new(Middleware::ClientScript)
				.with_pattern("*.client.lua")
				.with_child_pattern(".src.client.lua")
				.with_suffix(".client.lua")
				.with_exclude("init.client.lua"),
			SyncRule::new(Middleware::ModuleScript)
				.with_pattern("*.lua")
				.with_child_pattern(".src.lua")
				.with_exclude("init.lua"),
			//////////////////////////////////////////////////////////////////////////////////////////
			// Other file types
			SyncRule::new(Middleware::StringValue)
				.with_pattern("*.txt")
				.with_child_pattern("init.txt"),
			SyncRule::new(Middleware::RichStringValue)
				.with_pattern("*.md")
				.with_child_pattern("init.md"),
			SyncRule::new(Middleware::LocalizationTable)
				.with_pattern("*.csv")
				.with_child_pattern("init.csv"),
			SyncRule::new(Middleware::JsonModule)
				.with_pattern("*.json")
				.with_child_pattern("init.json")
				.with_excludes(&["*.model.json", "*.data.json", "*.meta.json"]),
			SyncRule::new(Middleware::TomlModule)
				.with_pattern("*.toml")
				.with_child_pattern("init.toml"),
			SyncRule::new(Middleware::YamlModule)
				.with_pattern("*.yaml")
				.with_child_pattern("init.yaml"),
			SyncRule::new(Middleware::YamlModule)
				.with_pattern("*.yml")
				.with_child_pattern("init.yml"),
			SyncRule::new(Middleware::MsgpackModule)
				.with_pattern("*.msgpack")
				.with_child_pattern("init.msgpack"),
			// Model files
			SyncRule::new(Middleware::JsonModel)
				.with_pattern("*.model.json")
				.with_child_pattern("init.model.json")
				.with_suffix(".model.json"),
			SyncRule::new(Middleware::RbxmModel)
				.with_pattern("*.rbxm")
				.with_child_pattern("init.rbxm"),
			SyncRule::new(Middleware::RbxmxModel)
				.with_pattern("*.rbxmx")
				.with_child_pattern("init.rbxmx"),
		]
	})
}
