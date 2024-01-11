use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Get)]
pub fn derive_index(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = &input.ident;

	let arms = match input.data {
		Data::Struct(data) => {
			let mut arms = TokenStream::new();

			match data.fields {
				Fields::Named(named) => {
					for field in named.named {
						match field.ident {
							Some(ident) => {
								let index = ident.to_string();

								let arm = quote! {
									#index => Some(&self.#ident),
								};

								arms.extend(arm);
							}
							None => unimplemented!("Tuples are not supported"),
						}
					}
				}
				_ => unimplemented!("Only named fields are supported"),
			}

			arms
		}
		_ => {
			unimplemented!("Only flat structs are supported")
		}
	};

	let expanded = quote! {
		impl #name {
			fn get(&self, index: &str) -> Option<&i32> {
				match index {
					#arms
					_ => None,
				}
			}
		}
	};

	proc_macro::TokenStream::from(expanded)
}
