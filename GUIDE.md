# Rust Style Guide

## Formatting conventions

These formatting conventions are a work in progress, and may do anything they
like, up to and including eating your laundry.

### Function definitions

In Rust, one finds functions by searching for `fn [function-name]`; It's
important that you style your code so that it's very searchable in this way.

The proper ordering and spacing is:

```rust
[pub] [unsafe] [extern ["ABI"]] fn foo(arg1: i32, arg2: i32) -> i32 {
    ...
}
```

Avoid comments within the signature itself.

### Closures

Don't put any extra spaces before the first `|` (unless the closure is prefixed
by `move`), but put a space between the second `|` and the expression of the
closure. Between the `|`s, you should use function definition syntax, however,
elide types where possible.

Use closures without the enclosing `{}`, if possible. Add the `{}` when you
have a return type, when there are statements before the final expression, or
when you need to split it into multiple lines; examples:

```rust
|arg1, arg2| expr

move |arg1: i32, arg2: i32| -> i32 { expr1; expr2 }
```

### Indentation

Use spaces, not tabs.


### Expressions

Do not include a space between a unary op and its operand (i.e., `!x`, not
`! x`).

Do include spaces around binary ops (i.e., `x + 1`, not `x+1`) (including `=`).

For comparison operators, because for `T op U`, `&T op &U` is also implemented:
if you have `t: &T`, and `u: U`, prefer `*t op u` to `t op &u`. In general,
within expressions, prefer dereferencing to taking references.

Do not include extraneous parentheses for `if` and `while` expressions.

```rust
if true {
}
```

is better than

```rust
if (true) {
}
```

Do include extraneous parentheses if it makes an arithmetic or logic expression
easier to understand (`(x * 15) + (y * 20)` is fine)

### Function calls

Do not put a space between the function name, and the opening parenthesis.

Do not put a space between an argument, and the comma which follows.

Do put a space between an argument, and the comma which precedes it.

#### Single-line calls

Do not put a space between the function name and open paren, between the open
paren and the first argument, or between the last argument and the close paren.

Do not put a comma after the last argument.

```rust
foo(x, y, z)
```

### Methods

Follow the function rules for calling.

#### Single-line

Do not put any spaces around the `.`.

```rust
x.foo().bar().baz(x, y, z);
```

### as

Put spaces before and after `as`:

```rust
let cstr = "Hi\0" as *const str as *const [u8] as *const std::os::raw::c_char;
```

### Tuples and tuple structs

Write the type list as you would a parameter list to a function.

Build a tuple or tuple struct as you would call a function.

#### Single-line

```rust
struct Bar(Type1, Type2);

let x = Bar(11, 22);
let y = (11, 22, 33);
```

### Enums

In the declaration, put each variant on its own line.

Format each variant accordingly as either a `struct`, `tuple struct`, or ident,
which doesn't require special formatting.

```rust
enum FooBar {
    First(u32),
    Second,
    Error {
        err: Box<Error>,
        line: u32,
    },
}
```

### macro\_rules!

Use `{}` for the full definition of the macro.

```rust
macro_rules! foo {
}
```

### Doc comments

Prefer outer doc comments (`///` or `//*`), only use inner doc comments (`//!`
and `/*!`) to write module-level documentation.

### Attributes

Put each attribute on its own line, indented to the indentation of its item.
In the case of inner attributes (`#!`), indent it to the inner indentation (the
indentation of the item + 1). Prefer outer attributes, where possible.

For attributes with argument lists, format like functions.

```rust
#[repr(C)]
#[foo(foo, bar)]
struct CRepr {
    #![repr(C)]
    x: f32,
    y: f32,
}
```

### Extern crate

`extern crate foo;`

Use spaces around keywords, no spaces around the semi-colon.

### Modules

```rust
mod foo {
}
```

```rust
mod foo;
```

Use spaces around keywords and before the opening brace, no spaces around the
semi-colon.

## Non-formatting conventions

### Expressions

Prefer to use Rust's expression oriented nature where possible;

```rust
// use
let x = if y { 1 } else { 0 };
// not
let x;
if y {
    x = 1;
} else {
    x = 0;
}
```

### Names

 * Types shall be `PascalCase`,
 * Enum variants shall be `PascalCase`,
 * Struct members shall be `snake_case`,
 * Function and method names shall be `snake_case`,
 * Local variables shall be `snake_case`,
 * Macro names shall be `snake_case`,
 * Constants (`const`s and immutable `static`s) shall be `SCREAMING_SNAKE_CASE`.

### Modules

Avoid `#[path]` annotations where possible.

## Cargo.toml conventions

### Formatting conventions

Use the same line width and indentation as Rust code.

Put a blank line between the last key-value pair in a section and the header of
the next section. Do not place a blank line between section headers and the
key-value pairs in that section, or between key-value pairs in a section.

Sort key names alphabetically within each section, with the exception of the
`[package]` section. Put the `[package]` section at the top of the file; put
the `name` and `version` keys in that order at the top of that section,
followed by the remaining keys other than `description` in alphabetical order,
followed by the `description` at the end of that section.

Don't use quotes around any standard key names; use bare keys. Only use quoted
keys for non-standard keys whose names require them, and avoid introducing such
key names when possible.  See the [TOML
specification](https://github.com/toml-lang/toml/blob/master/versions/en/toml-v0.4.0.md#table)
for details.

Put a single space both before and after the `=` between a key and value. Do
not indent any key names; start all key names at the start of a line.

Use multi-line strings (rather than newline escape sequences) for any string
values that include multiple lines, such as the crate description.

For array values, such as a list of authors, put the entire list on the same
line as the key, if it fits. Otherwise, use block indentation: put a newline
after the opening square bracket, indent each item by one indentation level,
put a comma after each item (including the last), and put the closing square
bracket at the start of a line by itself after the last item.

```
authors = [
    "A Uthor <a.uthor@example.org>",
    "Another Author <author@example.net>",
]
```

For table values, such as a crate dependency with a path, write the entire
table using curly braces and commas on the same line as the key if it fits. If
the entire table does not fit on the same line as the key, separate it out into
a separate section with key-value pairs:

```
[dependencies]
crate1 = { path = "crate1", version = "1.2.3" }

[dependencies.extremely_long_crate_name_goes_here]
path = "extremely_long_path_name_goes_right_here"
version = "4.5.6"
```

### Metadata conventions

The authors list should consist of strings that each contain an author name
followed by an email address in angle brackets: `Full Name <email@address>`.
It should not contain bare email addresses, or names without email addresses.
(The authors list may also include a mailing list address without an associated
name.)

The license field must contain a valid [SPDX
expression](https://spdx.org/spdx-specification-21-web-version#h.jxpfx0ykyb60),
using valid [SPDX license names](https://spdx.org/licenses/). (As an exception,
by widespread convention, the license field may use `/` in place of ` OR `; for
example, `MIT/Apache-2.0`.)

The homepage field, if present, must consist of a single URL, including the
scheme (e.g. `https://example.org/`, not just `example.org`.)

Within the description field, wrap text at 78 columns. Don't start the
description field with the name of the crate (e.g. "cratename is a ..."); just
describe the crate itself. If providing a multi-sentence description, the first
sentence should go on a line by itself and summarize the crate, like the
subject of an email or commit message; subsequent sentences can then describe
the crate in more detail.
