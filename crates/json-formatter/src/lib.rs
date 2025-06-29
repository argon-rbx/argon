use serde_json::ser::Formatter;
use std::io;

macro_rules! tri {
	($e:expr $(,)?) => {
		match $e {
			core::result::Result::Ok(val) => val,
			core::result::Result::Err(err) => return core::result::Result::Err(err),
		}
	};
}

/// This structure pretty prints a JSON value to make it human readable.
#[derive(Clone, Debug)]
pub struct JsonFormatter<'a> {
	current_indent: usize,
	has_value: bool,
	indent: &'a [u8],
	array_breaks: bool,
	extra_newline: bool,
	max_decimals: usize,
}

impl<'a> JsonFormatter<'a> {
	/// Construct a pretty printer formatter that defaults to using two spaces for indentation.
	pub fn new() -> Self {
		JsonFormatter {
			current_indent: 0,
			has_value: false,
			indent: b"  ",
			array_breaks: true,
			extra_newline: false,
			max_decimals: 0,
		}
	}

	/// Construct a pretty printer formatter that uses the `indent` string for indentation.
	pub fn with_indent(mut self, indent: &'a [u8]) -> Self {
		self.indent = indent;
		self
	}

	/// Construct a pretty printer formatter that optionally break arrays into multiple lines.
	pub fn with_array_breaks(mut self, array_breaks: bool) -> Self {
		self.array_breaks = array_breaks;
		self
	}

	/// Construct a pretty printer formatter that adds an extra newline at the end.
	pub fn with_extra_newline(mut self, extra_newline: bool) -> Self {
		self.extra_newline = extra_newline;
		self
	}

	/// Construct a pretty printer formatter that limits the number of decimal places.
	pub fn with_max_decimals(mut self, max_decimals: usize) -> Self {
		self.max_decimals = max_decimals;
		self
	}
}

impl<'a> Default for JsonFormatter<'a> {
	fn default() -> Self {
		JsonFormatter::new()
	}
}

impl<'a> Formatter for JsonFormatter<'a> {
	#[inline]
	fn write_f64<W>(&mut self, writer: &mut W, mut value: f64) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		if self.max_decimals > 0 {
			let multiplier = 10_f64.powi(self.max_decimals as i32);
			value = (value * multiplier).round() / multiplier;
		}

		let mut buffer = ryu::Buffer::new();
		let s = buffer.format_finite(value);
		writer.write_all(s.as_bytes())
	}

	#[inline]
	fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		self.current_indent += 1;
		self.has_value = false;
		writer.write_all(b"[")
	}

	#[inline]
	fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		self.current_indent -= 1;

		if self.has_value && self.array_breaks {
			tri!(writer.write_all(b"\n"));
			tri!(indent(writer, self.current_indent, self.indent));
		}

		writer.write_all(b"]")
	}

	#[inline]
	fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		if !self.array_breaks {
			if !first {
				tri!(writer.write_all(b", "));
			}

			return Ok(());
		}

		tri!(writer.write_all(if first { b"\n" } else { b",\n" }));
		indent(writer, self.current_indent, self.indent)
	}

	#[inline]
	fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		self.has_value = true;
		Ok(())
	}

	#[inline]
	fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		self.current_indent += 1;
		self.has_value = false;
		writer.write_all(b"{")
	}

	#[inline]
	fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		self.current_indent -= 1;

		if self.has_value {
			tri!(writer.write_all(b"\n"));
			tri!(indent(writer, self.current_indent, self.indent));
		}

		tri!(writer.write_all(b"}"));

		if self.current_indent == 0 && self.extra_newline {
			writer.write_all(b"\n")
		} else {
			Ok(())
		}
	}

	#[inline]
	fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		tri!(writer.write_all(if first { b"\n" } else { b",\n" }));
		indent(writer, self.current_indent, self.indent)
	}

	#[inline]
	fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		writer.write_all(b": ")
	}

	#[inline]
	fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
	where
		W: ?Sized + io::Write,
	{
		self.has_value = true;
		Ok(())
	}
}

fn indent<W>(wr: &mut W, n: usize, s: &[u8]) -> io::Result<()>
where
	W: ?Sized + io::Write,
{
	for _ in 0..n {
		tri!(wr.write_all(s));
	}

	Ok(())
}
