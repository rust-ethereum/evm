use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
	LitStr, Token,
	parse::{Parse, ParseStream},
	parse_macro_input,
};

#[derive(Debug)]
struct Input {
	prefix: LitStr,
	folder: LitStr,
	flags: Vec<Ident>,
}

impl Parse for Input {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let prefix = input.parse()?;
		input.parse::<Token![,]>()?;
		let folder = input.parse()?;

		if input.is_empty() {
			return Ok(Input {
				prefix,
				folder,
				flags: Vec::new(),
			});
		}

		input.parse::<Token![;]>()?;
		let flags = input
			.parse_terminated(Ident::parse, Token![,])?
			.into_iter()
			.collect();
		Ok(Input {
			prefix,
			folder,
			flags,
		})
	}
}

const TIME_CONSUMING_TESTS: [&str; 6] = [
	"legacytests_cancun__stTimeConsuming",
	"legacytests_constaninople__stTimeConsuming",
	"legacytests_cancun__VMTests__vmPerformance",
	"legacytests_constaninople__VMTests__vmPerformance",
	"oldethtests__stTimeConsuming",
	"oldethtests__VMTests__vmPerformance",
];

#[proc_macro]
pub fn statetest_folder(item: TokenStream) -> TokenStream {
	let mut source_file = std::path::absolute(
		proc_macro::Span::call_site()
			.local_file()
			.expect("can only handle local file"),
	)
	.expect("cannot get absolute path");
	assert!(source_file.pop());
	let input = parse_macro_input!(item as Input);
	let prefix = input.prefix.value();
	let folder = source_file
		.join(input.folder.value())
		.canonicalize()
		.expect("failed to canonicalize");
	let test_files =
		jsontests::run::collect_test_files(folder.to_str().expect("non-utf8 filename"))
			.expect("uninitialized submodule");

	let mut disable_eip7610 = false;
	for flag in input.flags {
		if flag == "disable_eip7610" {
			disable_eip7610 = true;
		} else {
			panic!("unknown flag");
		}
	}

	let mut tests = Vec::new();
	for (test_file_name, test_file_path) in test_files {
		let mut name = format!("{prefix:}/{test_file_name:}");
		name = name.replace("/", "__");
		let ident = Ident::new(&name, Span::call_site());
		let modify_config = if disable_eip7610 {
			quote! { jsontests::run::disable_eip7610 }
		} else {
			quote! { jsontests::run::empty_config_change }
		};
		let time_consuming_flag = if TIME_CONSUMING_TESTS.iter().any(|t| name.starts_with(t)) {
			quote! { #[cfg(feature = "time-consuming")] }
		} else {
			quote! {}
		};
		tests.push(quote! {
			#time_consuming_flag
			#[test]
			fn #ident() -> Result<(), jsontests::error::Error> {
				let tests_status = jsontests::run::run_file(#test_file_path, false, None, #modify_config);
				match tests_status {
					Ok(tests_status) => {
						tests_status.print_total();
						Ok(())
					},
					Err(err) => {
						// Rerun tests with debug flag.
						jsontests::run::run_file(#test_file_path, true, Some("failed.json"), #modify_config)?;
						Err(err)
					},
				}
			}
		})
	}

	quote! {
		#( #tests )*
	}
	.into()
}
