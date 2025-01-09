use regex::{Captures, Regex};

pub struct Parser {
	markdown_patterns: Vec<&'static str>,
	markdown_syntax: Vec<Option<&'static str>>,
	rich_text_syntax: Vec<(&'static str, &'static str)>,
}

impl Parser {
	pub fn new() -> Self {
		Self {
			markdown_patterns: vec![
				r"(?m)^(#{1,6})\s+(.*)$",    // Heading pattern
				r"\*\*\*[\w\p{P}\s]+\*\*\*", // Bold + Italics
				r"\*\*[\w\p{P}\s]+\*\*",     // Bold
				r"\*[\w\p{P}\s]+\*",         // Italics
				r"~~[\w\p{P}\s]+~~",         // Strikethrough
				r"__[\w\p{P}\s]+__",         // Underline
				r"`[\w\p{P}\s]+`",           // Inline code
			],
			markdown_syntax: vec![
				None,        // Heading doesn't have direct symbols to remove
				Some("***"), // Bold + Italics
				Some("**"),  // Bold
				Some("*"),   // Italics
				Some("~~"),  // Strikethrough
				Some("`"),   // Inline code
				Some("__"),  // Underline
			],
			rich_text_syntax: vec![
				("<b>", "</b>"),                                                          // Heading -> Bold
				("<b><i>", "</i></b>"),                                                   // Bold + Italics
				("<b>", "</b>"),                                                          // Bold
				("<i>", "</i>"),                                                          // Italics
				("<s>", "</s>"),                                                          // Strikethrough
				("<u>", "</u>"),                                                          // Underline
				("<font family='rbxasset://fonts/families/RobotoMono.json'>", "</font>"), // Inline code
			],
		}
	}

	pub fn parse(&self, text: &str) -> String {
		let mut result = text.to_string();

		for (index, pattern) in self.markdown_patterns.iter().enumerate() {
			let regex = Regex::new(pattern).unwrap();

			let (start, end) = self.rich_text_syntax[index];

			result = regex
				.replace_all(&result, |caps: &Captures| {
					if index == 0 {
						let heading = caps.get(2).or_else(|| caps.get(0)).unwrap().as_str();
						return format!("{start}{heading}{end}");
					}

					let content = caps.get(0).unwrap().as_str();

					if let Some(syntax) = self.markdown_syntax[index] {
						let cleaned = content.replace(syntax, "");
						format!("{start}{cleaned}{end}")
					} else {
						format!("{start}{content}{end}")
					}
				})
				.to_string();
		}

		result
	}
}
