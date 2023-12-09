pub struct UpdateMeta {
	name: String,
	game_id: Option<i64>,
	place_ids: Option<Vec<u64>>,
}

pub struct Sync {
	action: MessageAction,
	path: String,
	kind: Option<String>,
	// data: Option<String>,
}

pub enum MessageAction {
	Create,
	Update,
	Delete,
	Write,
}
