#[derive(Debug)]
pub enum Message {
	UpdateMeta(UpdateMeta),
	Sync(Sync),
}

#[derive(Debug)]
pub struct UpdateMeta {
	pub name: String,
	pub game_id: Option<i64>,
	pub place_ids: Option<Vec<u64>>,
}

#[derive(Debug)]
pub struct Sync {
	pub action: MessageAction,
	pub path: String,
	pub kind: Option<String>,
	// pub data: Option<String>,
}

#[derive(Debug)]
pub enum MessageAction {
	Create,
	Update,
	Delete,
	Write,
}
