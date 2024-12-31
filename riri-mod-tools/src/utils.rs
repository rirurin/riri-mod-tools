#![allow(dead_code)]

use std::{
    error::Error,
    fmt::Write,
    path::{ Path, PathBuf }
};
use walkdir::DirEntry;

fn check_file_extension(f: &DirEntry, ext: &str) -> bool {
    let a: &std::path::Path = f.file_name().as_ref();
    let fext = match a.extension() {
        Some(v) => v,
        None => return false
    };
    f.file_type().is_file() && fext.to_str().unwrap() == ext
}

#[inline(always)]
pub(crate) fn is_rust_source(f: &DirEntry) -> bool { check_file_extension(f, "rs") }
#[inline(always)]
pub(crate) fn is_csharp_source(f: &DirEntry) -> bool { check_file_extension(f, "cs") }

pub(crate) fn get_or_make_child_dir<T: AsRef<Path>>(d: T, c: &str) -> Result<PathBuf, Box<dyn Error>> {
    let out = d.as_ref().join(c);
    if !out.exists() { std::fs::create_dir(&out)?; }
    Ok(out)
}

#[derive(Debug)]
pub struct SourceWriterError(String);
impl Error for SourceWriterError { }
impl std::fmt::Display for SourceWriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error when using SourceWriter: {}", self.0)
    }
}

pub struct SourceWriter {
    buffer: String,
    indentation_level: usize,
    indent_type: String
}
const INDENT_LIMIT: usize = 128;
const WRITER_DEFAULT_CAP: usize = 1024;
impl SourceWriter {
    pub fn new() -> Self {
        Self::new_inner("    ".to_owned(), WRITER_DEFAULT_CAP)
    }
    pub fn new_custom_indent(indent_type: String) -> Self {
        Self::new_inner(indent_type, WRITER_DEFAULT_CAP)
    }
    fn new_inner(indent_type: String, cap: usize) -> Self {
        Self {
            buffer: String::with_capacity(cap),
            indentation_level: 0,
            indent_type
        }
    }
    pub fn fmt(&mut self, args: std::fmt::Arguments) -> Result<(), std::fmt::Error> {
        match self.indentation_level {
            0 => (),
            1 => self.buffer.push_str(&self.indent_type),
            _ => self.buffer.push_str(&self.indent_type.repeat(self.indentation_level))
        };
        self.buffer.write_fmt(args)
    }
    pub fn fmtln(&mut self, args: std::fmt::Arguments) -> Result<(), std::fmt::Error> {
        self.fmt(args)?;
        self.buffer.push_str("\n");
        Ok(())
    }
    pub fn write(&mut self, text: &str) {
        match self.indentation_level {
            0 => (),
            1 => self.buffer.push_str(&self.indent_type),
            _ => self.buffer.push_str(&self.indent_type.repeat(self.indentation_level))
        };
        self.buffer.push_str(text);
    }
    pub fn writeln(&mut self, text: &str) {
        self.write(text);
        self.buffer.push_str("\n");
    }
    pub fn indent(&mut self) -> Result<(), SourceWriterError> {
        if self.indentation_level + 1 > INDENT_LIMIT {
            Err(SourceWriterError("SourceWriter exceeded indentation limit".to_owned()))
        } else {
            self.writeln("\x7b");
            self.indentation_level += 1;
            Ok(())
        }
    }
    pub fn unindent(&mut self) -> Result<(), SourceWriterError> {
        if self.indentation_level == 0 {
            Err(SourceWriterError("Can't unindent SourceWriter, already at zero".to_owned()))
        } else {
            self.indentation_level -= 1;
            self.writeln("\x7d");
            Ok(())
        }
    }
    pub fn submit(self) -> String { self.buffer }
}

#[derive(Debug)]
pub struct GetGenericTypeError;
impl Error for GetGenericTypeError { }
impl std::fmt::Display for GetGenericTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error trying to obtain inner type")
    }
}

// Get the inner type from a type with a generic. Hooked statics generate a
// std::sync::OnceLock<T> -> T, and we're trying to obtain T.
pub(crate) fn generic_type_get_inner(outer: &syn::Type) -> Result<Option<&syn::Type>, Box<dyn Error>> {
    if let syn::Type::Path(p) = outer {
        // check last part of tail, since we expect the type argument to be here
        // NOTE: We're not expecting fully qualified paths here!
        // Also, the inner generic should only define one type!
        let path_tail = p.path.segments.last().unwrap();
        match &path_tail.arguments {
            syn::PathArguments::AngleBracketed(a) => {
                match a.args.first().unwrap() {
                    syn::GenericArgument::Type(t) => Ok(Some(t)),
                    _ => Err(Box::new(GetGenericTypeError))
                }
            },
            _ => Ok(None)
        }
    } else {
        Err(Box::new(GetGenericTypeError))
    }
}
