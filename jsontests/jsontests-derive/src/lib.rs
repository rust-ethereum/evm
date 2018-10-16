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

use attr::{Config, extract_attrs};
use tests::read_tests_from_dir;
use util::*;

use failure::Error;
use itertools::Itertools;

use syn::Ident;
use proc_macro::TokenStream;

#[proc_macro_derive(JsonTests, attributes(directory, test_with, bench_with, criterion_config, skip, should_panic, patch))]
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
    gen.parse().unwrap()
}

fn impl_json_tests(ast: &syn::DeriveInput) -> Result<quote::Tokens, Error> {
    let config = extract_attrs(&ast)?;
    let tests = read_tests_from_dir(&config.directory)?;
    let mut tokens = quote::Tokens::new();
    let mut bench_idents = Vec::new();

    // split tests into groups by filepath
    let tests = tests.group_by(|test| test.path.clone());

    let dir_mod_name = open_directory_module(&config, &mut tokens);

    // If behchmarking support is requested, import Criterion
    if config.bench_with.is_some() {
        tokens.append(quote! {
            use criterion::Criterion;
        })
    }

    for (filepath, tests) in &tests {
        // If tests count in this file is 1, we don't need submodule
        let tests = tests.collect::<Vec<_>>();
        let need_file_submodule = tests.len() > 1;
        let mut file_mod_name = None;
        if need_file_submodule {
            file_mod_name = Some(open_file_module(&filepath, &mut tokens));
            // If behchmarking support is requested, import Criterion
            if config.bench_with.is_some() {
                tokens.append(quote! {
                    use criterion::Criterion;
                })
            }
        }

        // Generate test function
        for test in tests {
            let name = sanitize_ident(&test.name);
            let name_ident = Ident::from(name.as_ref());
            let data = json::to_string(&test.data)?;

            generate_test(&config, &name_ident, &data, &mut tokens);
            generate_bench(&config, &name_ident, &data, &mut tokens)
                .map(|mut ident| {
                    // prepend dir submodule
                    ident = Ident::from(format!("{}::{}", dir_mod_name, ident.as_ref()));
                    // prepend file submodule
                    if need_file_submodule {
                        ident = Ident::from(format!("{}::{}", file_mod_name.as_ref().unwrap(), ident.as_ref()));
                    }
                    bench_idents.push(ident);
                });

        }

        if need_file_submodule {
            // Close file module
            close_brace(&mut tokens)
        }
    }

    // Close directory module
    close_brace(&mut tokens);

    generate_criterion_macros(&config, &bench_idents, &mut tokens);

    Ok(tokens)
}

fn generate_test(config: &Config, test_name: &Ident, data: &str, tokens: &mut quote::Tokens) {
    let test_func_path = &config.test_with.path;
    let test_func_name = &config.test_with.name;
    let test_name_str = test_name.as_ref();
    let (patch_name, patch_path) = derive_patch(config);

    tokens.append(quote!{#[test]});
    if config.should_panic {
        tokens.append(quote!{#[should_panic]});
    }

    tokens.append(quote! {
        fn #test_name() {
            use #test_func_path;
            use #patch_path;
            let data = #data;
            #test_func_name::<#patch_name>(#test_name_str, data);
        }
    });
}

fn generate_bench(config: &Config, test_name: &Ident, data: &str, tokens: &mut quote::Tokens) -> Option<Ident> {
    if config.bench_with.is_none() {
        return None
    }

    let bench = config.bench_with.as_ref().unwrap();
    let bench_func_path = &bench.path;
    let bench_func_name = &bench.name;

    let bench_name = format!("bench_{}", test_name.as_ref());
    let bench_ident = Ident::from(bench_name.as_ref());

    let (patch_name, patch_path) = derive_patch(config);

    tokens.append(quote! {
        pub fn #bench_ident(c: &mut Criterion) {
            use #bench_func_path;
            use #patch_path;
            let data = #data;
            #bench_func_name::<#patch_name>(c, #bench_name, data);
        }
    });

    Some(bench_ident)
}

fn generate_criterion_macros(config: &Config, benches: &[Ident], tokens: &mut quote::Tokens) {
    // Generate criterion macros
    if config.bench_with.is_some() {
        let benches = benches.iter().map(AsRef::as_ref).join(" , ");
        let config = config.criterion_config
            .as_ref()
            .map(|cfg| cfg.path.clone())
            .unwrap_or_else(|| Ident::from("Criterion::default"));
        let template = quote! {
            criterion_group! {
                name = main;
                config = #config();
                targets = TARGETS
            };
        };
        tokens.append(template.as_ref().replace("TARGETS", &benches));
    }
}

fn derive_patch(config: &Config) -> (Ident, Ident) {
    if let Some(patch) = config.patch.as_ref() {
        (patch.name.clone(), patch.path.clone())
    } else {
        (
            Ident::from("VMTestPatch"),
            Ident::from("evm::VMTestPatch")
        )
    }
}
