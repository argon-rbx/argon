use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse, parse_macro_input, Item, LitStr};

#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut item = parse_macro_input!(item as Item);

	let item_fn = match &mut item {
		Item::Fn(item_fn) => item_fn,
		_ => panic!("expected function"),
	};

	let data = parse_macro_input!(attr as Option<LitStr>);
	let puffin_macro = quote! {
		puffin::profile_function!(#data);
	};

	item_fn
		.block
		.stmts
		.insert(0, parse(puffin_macro.into()).unwrap());

	item.into_token_stream().into()
}
