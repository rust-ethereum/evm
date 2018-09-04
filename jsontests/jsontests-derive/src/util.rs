use syn::Ident;
use quote;

use attr::Config;

pub fn open_directory_module(config: &Config, tokens: &mut quote::Tokens) -> String {
    // get the leaf directory name
    let dirname = config.directory.rsplit('/').next().unwrap();

    // create identifier
    let dirname = sanitize_ident(dirname);
    let dirname_ident = Ident::from(dirname.as_ref());

    open_module(dirname_ident, tokens);

    dirname
}

pub fn open_file_module(filepath: &str, tokens: &mut quote::Tokens) -> String {
    // get file name without extension
    let filename = filepath.rsplit('/').next().unwrap()
        .split('.').next().unwrap();
    // create identifier
    let filename = sanitize_ident(filename);
    let filename_ident = Ident::from(filename.as_ref());

    open_module(filename_ident, tokens);

    filename
}

pub fn open_module<I: Into<Ident>>(module_name: I, tokens: &mut quote::Tokens) {
    let module_name = module_name.into();
    // append module opening tokens
    tokens.append(quote! {
        mod #module_name
    });
    tokens.append("{");
}

pub fn close_brace(tokens: &mut quote::Tokens) {
    tokens.append("}")
}

pub fn sanitize_ident(ident: &str) -> String {
    // replace empty ident
    let ident = if ident.is_empty() {
        String::from("unnamed")
    } else { ident.to_string() };

    // prepend alphabetic character if token starts with non-alphabetic character
    let ident = if ident.chars().nth(0).map(|c| !c.is_alphabetic()).unwrap_or(true) {
        format!("x_{}", ident)
    } else { ident };

    // replace special characters with _
    replace_chars(&ident, "!@#$%^&*-+=/<>;\'\"()`~", "_")
}

pub fn replace_chars(s: &str, from: &str, to: &str) -> String {
    let mut initial = s.to_owned();
    for c in from.chars() {
        initial = initial.replace(c, to);
    }
    initial
}
