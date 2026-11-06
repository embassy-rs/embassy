use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;

/// A location within the assembly string, not a Rust token span.
#[derive(Clone, Copy)]
pub(super) struct Location {
    pub line: usize,
    pub column: usize,
}

pub(super) struct Source<'a> {
    pub name: &'a str,
    pub span: Span,
}

impl Source<'_> {
    pub fn error(&self, location: Location, message: impl std::fmt::Display) -> syn::Error {
        syn::Error::new(
            self.span,
            format!("{}:{}:{}: {message}", self.name, location.line, location.column),
        )
    }
}

pub(super) struct Statement {
    pub mnemonic: String,
    pub operands: Vec<String>,
    pub location: Location,
}

pub(super) struct Assembly {
    pub statements: Vec<Statement>,
    pub labels: BTreeMap<String, u32>,
    pub entries: Vec<(String, u32)>,
}

pub(super) fn identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Read a signed integer without truncation. Operand-specific checks follow.
pub(super) fn integer(value: &str) -> Result<i64, &'static str> {
    let (negative, digits) = match value.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, value),
    };
    let (radix, digits) = if let Some(digits) = digits.strip_prefix("0x").or_else(|| digits.strip_prefix("0X")) {
        (16, digits)
    } else if let Some(digits) = digits.strip_prefix("0b").or_else(|| digits.strip_prefix("0B")) {
        (2, digits)
    } else {
        (10, digits)
    };
    if digits.is_empty()
        || digits.starts_with('_')
        || digits.ends_with('_')
        || !digits.chars().all(|c| c == '_' || c.is_digit(radix))
    {
        return Err("expected a decimal, hexadecimal (0x), or binary (0b) integer");
    }
    let digits = digits.replace('_', "");
    let magnitude = i64::from_str_radix(&digits, radix).map_err(|_| "invalid or overflowing integer")?;
    if negative {
        magnitude.checked_neg().ok_or("overflowing integer")
    } else {
        Ok(magnitude)
    }
}

pub(super) fn parse(text: &str, source: &Source<'_>) -> syn::Result<Assembly> {
    let mut statements = Vec::new();
    let mut labels = BTreeMap::new();
    let mut exports = Vec::new();
    let mut export_names = BTreeSet::new();
    let mut word_offset = 0u32;

    for (line_number, original) in text.lines().enumerate() {
        let comment = [original.find("//"), original.find(';')].into_iter().flatten().min();
        let line = &original[..comment.unwrap_or(original.len())];
        let mut statement = line.trim();
        let mut location = Location {
            line: line_number + 1,
            column: line.len() - line.trim_start().len() + 1,
        };
        if statement.is_empty() {
            continue;
        }
        if !statement.is_ascii() {
            return Err(source.error(location, "assembly statements must use ASCII characters"));
        }

        if let Some((label, rest)) = statement.split_once(':') {
            let label = label.trim();
            if !identifier(label) {
                return Err(source.error(location, "invalid label; expected an ASCII identifier"));
            }
            if labels.insert(label.to_owned(), word_offset).is_some() {
                return Err(source.error(location, format!("duplicate label `{label}`")));
            }
            location.column += statement.len() - rest.trim_start().len();
            statement = rest.trim();
            if statement.is_empty() {
                continue;
            }
        }

        let (mnemonic, operands) = statement
            .split_once(char::is_whitespace)
            .map_or((statement, ""), |(mnemonic, operands)| (mnemonic, operands.trim()));
        let mnemonic = mnemonic.to_ascii_lowercase();
        let operands: Vec<String> = if operands.is_empty() {
            Vec::new()
        } else {
            operands.split(',').map(|operand| operand.trim().to_owned()).collect()
        };
        if operands.iter().any(String::is_empty) {
            return Err(source.error(location, "empty operand"));
        }

        if mnemonic == ".entry" {
            if operands.len() != 1 || !identifier(&operands[0]) {
                return Err(source.error(location, "expected `.entry label`"));
            }
            let name = &operands[0];
            if !export_names.insert(name.clone()) {
                return Err(source.error(location, format!("duplicate entry `{name}`")));
            }
            exports.push((name.clone(), location));
            continue;
        }

        word_offset = word_offset
            .checked_add(1)
            .ok_or_else(|| source.error(location, "program exceeds the u32 word-offset limit"))?;
        statements.push(Statement {
            mnemonic,
            operands,
            location,
        });
    }

    if statements.is_empty() {
        return Err(source.error(Location { line: 1, column: 1 }, "program contains no words"));
    }
    let mut entries = Vec::new();
    for (name, location) in exports {
        let offset = labels
            .get(&name)
            .ok_or_else(|| source.error(location, format!("undefined entry label `{name}`")))?;
        if *offset >= word_offset {
            return Err(source.error(location, format!("entry `{name}` points past the image")));
        }
        entries.push((name, *offset));
    }
    Ok(Assembly {
        statements,
        labels,
        entries,
    })
}
