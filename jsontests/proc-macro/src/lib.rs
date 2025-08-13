use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::{LitStr, parse::{Parse, ParseStream}, parse_macro_input, Token};
use quote::quote;

#[derive(Debug)]
struct Input {
	prefix: LitStr,
	folder: LitStr,
}

impl Parse for Input {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let prefix = input.parse()?;
		input.parse::<Token![,]>()?;
		let folder = input.parse()?;
		Ok(Input {
			prefix,
			folder,
		})
	}
}

#[proc_macro]
pub fn statetest_folder(item: TokenStream) -> TokenStream {
	let mut source_file = std::path::absolute(proc_macro::Span::call_site().local_file().expect("can only handle local file")).expect("cannot get absolute path");
	assert!(source_file.pop());
	let input = parse_macro_input!(item as Input);
	let prefix = input.prefix.value();
	let folder = source_file.join(input.folder.value()).canonicalize().expect("failed to canonicalize");
	let test_files = jsontests::run::collect_test_files(&folder.to_str().expect("non-utf8 filename")).expect("uninitialized submodule");

	let mut tests = Vec::new();
	for (test_file_name, test_file_path) in test_files {
		let mut name = format!("{prefix:}/{test_file_name:}");
		name = name.replace("/", "__");
		let ident = Ident::new(&name, Span::call_site());
		tests.push(quote! {
			#[test]
			fn #ident() {
				let tests_status = jsontests::run::run_file(#test_file_path, false, None).expect("test failed");
				tests_status.print_total();
			}
		})
	}

	quote! {
		#( #tests )*
	}.into()
}
