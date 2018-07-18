#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;
extern crate serde_json as json;
#[macro_use]
extern crate failure;
extern crate itertools;

mod attr;
mod tests;
mod util;

use attr::extract_attrs;
use tests::read_tests_from_dir;
use util::*;

use failure::Error;
use itertools::Itertools;

use syn::Ident;
use proc_macro::TokenStream;

#[proc_macro_derive(JsonTests, attributes(directory, test_with, bench_with, skip, should_panic))]
pub fn json_tests(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = match impl_json_tests(&ast) {
        Ok(tokens) => tokens,
        Err(err) => panic!("{}", err)
    };

    // Return the generated impl
    let parsed = gen.parse().unwrap();

    parsed
}

fn impl_json_tests(ast: &syn::DeriveInput) -> Result<quote::Tokens, Error> {
    let config = extract_attrs(&ast)?;
    let tests = read_tests_from_dir(&config.directory)?;
    let mut tokens = quote::Tokens::new();

    // split tests into groups by filepath
    let tests = tests.into_iter().group_by(|test| test.path.clone());

    open_directory_module(&config, &mut tokens);

    for (filepath, tests) in &tests {
        // If tests count in this file is 1, we don't need submodule
        let tests = tests.collect::<Vec<_>>();
        let need_file_submodule = tests.len() > 1;

        if need_file_submodule {
            open_file_module(&filepath, &mut tokens);
        }

        // Generate test function
        for test in tests {
            let ref test_func_path = config.test_with.path;
            let ref test_func_name = config.test_with.name;
            let name = sanitize_ident(&test.name);
            let name_ident = Ident::from(name.as_ref());
            let data = json::to_string(&test.data)?;

            // generate test attrs
            tokens.append(quote!{#[test]});
            if config.should_panic {
                tokens.append(quote!{#[should_panic]});
            }

            // generate test body
            tokens.append(quote! {
                fn #name_ident() {
                    use #test_func_path;
                    let data = #data;
                    #test_func_name(#name, data);
                }
            });

            // generate optional benchmark body
            if let Some(ref bench) = config.bench_with {
                let ref bench_func_path = bench.path;
                let ref bench_func_name = bench.name;
                let name = format!("bench_{}", name);
                let name_ident = Ident::from(name.as_ref());

                tokens.append(quote! {
                    #[bench]
                    fn #name_ident(b: &mut test::Bencher) {
                        use #bench_func_path;
                        let data = #data;
                        #bench_func_name(b, #name, data);
                    }
                })
            }
        }

        if need_file_submodule {
            // Close file module
            close_brace(&mut tokens)
        }
    }

    // Close directory module
    close_brace(&mut tokens);

    Ok(tokens)
}



