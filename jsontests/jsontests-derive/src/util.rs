use syn::Ident;
use quote;

use attr::Config;

pub fn open_directory_module(config: &Config, tokens: &mut quote::Tokens) {
    // get the leaf directory name
    let dirname = config.directory.rsplit("/").next().unwrap();

    // create identifier
    let dirname = sanitize_ident(dirname);
    let dirname = Ident::from(dirname);

    open_module(dirname, tokens);
}

pub fn open_file_module(filepath: &str, tokens: &mut quote::Tokens) {
    // get file name without extension
    let filename = filepath.rsplit("/").next().unwrap()
        .split(".").next().unwrap();
    // create identifier
    let filename = sanitize_ident(filename);
    let filename = Ident::from(filename);

    open_module(filename, tokens);
}

pub fn open_module(module_name: Ident, tokens: &mut quote::Tokens) {
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
    let ident = replace_chars(&ident, "!@#$%^&*-+=/<>;\'\"()`~", "_");

    ident
}

pub fn replace_chars(s: &str, from: &str, to: &str) -> String {
    let mut initial = s.to_owned();
    for c in from.chars() {
        initial = initial.replace(c, to);
    }
    initial
}
