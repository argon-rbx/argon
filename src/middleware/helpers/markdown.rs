use markdown::{
	Block::{self, Blockquote, CodeBlock, Header, Hr, OrderedList, Paragraph, Raw, UnorderedList},
	ListItem,
	Span::{self, Break, Code, Emphasis, Image, Link, Strong, Text},
};

const LIST_ELEMENT_PREFIX: &str = "<b>•</b>";

fn escape(text: &str) -> String {
	text.replace("&", "&amp;")
		.replace("<", "&lt;")
		.replace("\"", "&quot;")
		.replace("'", "&#8217;")
		.replace(">", "&gt;")
}

fn format_spans(elements: &[Span]) -> String {
	let mut string = String::new();

	for element in elements.iter() {
		string.push_str(&match *element {
			Text(ref text) => escape(text),
			Strong(ref content) => format!("<b>{}</b>", format_spans(content)),
			Emphasis(ref content) => format!("<i>{}</i>", format_spans(content)),
			Code(ref text) => format!(
				"<font family='rbxasset://fonts/families/RobotoMono.json'>{}</font>",
				&escape(text)
			),
			Link(ref text, _, _) => escape(text),
			Image(_, _, _) => String::new(),
			Break => String::from("\n"),
		})
	}

	string
}

fn format_list(elements: &[ListItem], ordered: bool) -> String {
	let mut string = String::new();

	for (index, item) in elements.iter().enumerate() {
		let prefix = if ordered {
			format!("{}.", index + 1)
		} else {
			String::from(LIST_ELEMENT_PREFIX)
		};

		string.push_str(&format!(
			"{prefix} {}\n",
			match *item {
				ListItem::Simple(ref elements) => format_spans(elements),
				ListItem::Paragraph(ref elements) => walk(elements),
			}
		))
	}

	format!("{}\n", string)
}

fn format_header(elements: &[Span], _level: usize) -> String {
	format!("<b>{}</b>\n\n", format_spans(elements),)
}

fn format_paragraph(elements: &[Span]) -> String {
	format!("{}\n\n", format_spans(elements))
}

fn format_blockquote(elements: &[Block]) -> String {
	format!("<i>{}</i>\n\n", walk(elements))
}

fn format_codeblock(_lang: &Option<String>, elements: &str) -> String {
	format!(
		"<font family='rbxasset://fonts/families/RobotoMono.json'>{}</font>\n\n",
		&escape(elements)
	)
}

fn format_unordered_list(elements: &[ListItem]) -> String {
	format_list(elements, false)
}

fn format_ordered_list(elements: &[ListItem]) -> String {
	format_list(elements, true)
}

fn walk(blocks: &[Block]) -> String {
	let mut string = String::new();

	for block in blocks.iter() {
		string.push_str(&match *block {
			Header(ref elements, level) => format_header(elements, level),
			Paragraph(ref elements) => format_paragraph(elements),
			Blockquote(ref elements) => format_blockquote(elements),
			CodeBlock(ref lang, ref elements) => format_codeblock(lang, elements),
			UnorderedList(ref elements) => format_unordered_list(elements),
			OrderedList(ref elements, _) => format_ordered_list(elements),
			Raw(ref elements) => elements.to_owned(),
			Hr => String::from("\n\n"),
		})
	}

	string.trim().to_owned()
}

pub fn parse(text: &str) -> String {
	walk(&markdown::tokenize(text))
}
