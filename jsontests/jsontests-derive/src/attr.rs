use failure::Error;

use syn;
use syn::Ident;
use syn::Lit::Str;
use syn::MetaItem;

#[derive(Default)]
pub struct Config {
    pub directory: String,
    pub test_with: ExternalRef,
    pub bench_with: Option<ExternalRef>,
    pub criterion_config: Option<ExternalRef>,
    pub patch: Option<ExternalRef>,
    pub skip: bool,
    pub should_panic: bool,
}

#[derive(Clone, Debug)]
pub struct ExternalRef {
    pub path: Ident,
    pub name: Ident,
}

impl Default for ExternalRef {
    fn default() -> Self {
        ExternalRef {
            path: Ident::from(String::new()),
            name: Ident::from(String::new()),
        }
    }
}

impl From<String> for ExternalRef {
    fn from(path: String) -> Self {
        let name = path.rsplit(':').next().unwrap().to_owned();
        ExternalRef {
            path: Ident::from(path),
            name: Ident::from(name),
        }
    }
}

pub fn extract_attrs(ast: &syn::DeriveInput) -> Result<Config, Error> {
    const ERROR_MSG: &str = "expected 2 attributes and 3 optional\n\n\
                             #[derive(JsonTests)]\n\
                             #[directory = \"../tests/testset\"]\n\
                             #[test_with = \"test::test_function\"]\n\
                             #[bench_wuth = \"test::bench_function\"] (Optional)\n\
                             #[patch = \"CustomTestPatch\" (Optional)\n\
                             #[skip] (Optional)\n\
                             #[should_panic] (Optional)\n\
                             struct TestSet;";

    if ast.attrs.len() < 2 || ast.attrs.len() > 6 {
        return Err(failure::err_msg(ERROR_MSG));
    }

    let config = ast
        .attrs
        .iter()
        .fold(Config::default(), |config, attr| match attr.value {
            MetaItem::NameValue(ref ident, Str(ref value, _)) => match ident.as_ref() {
                "directory" => Config {
                    directory: value.clone(),
                    ..config
                },
                "test_with" => Config {
                    test_with: ExternalRef::from(value.clone()),
                    ..config
                },
                "bench_with" => Config {
                    bench_with: Some(ExternalRef::from(value.clone())),
                    ..config
                },
                "criterion_config" => Config {
                    criterion_config: Some(ExternalRef::from(value.clone())),
                    ..config
                },
                "patch" => Config {
                    patch: Some(ExternalRef::from(value.clone())),
                    ..config
                },
                _ => panic!("{}", ERROR_MSG),
            },
            MetaItem::Word(ref ident) => match ident.as_ref() {
                "skip" => Config { skip: true, ..config },
                "should_panic" => Config {
                    should_panic: true,
                    ..config
                },
                _ => panic!("{}", ERROR_MSG),
            },
            _ => panic!("{}", ERROR_MSG),
        });

    // Check for incompatible options
    if config.should_panic && config.bench_with.is_some() {
        panic!("#[should_panic] is incompatible with benchmark tests");
    }

    Ok(config)
}
