//! dotenv
//!
//! A tiny, dependency-free utility to load environment variables from a
//! `.env` file located at your project root (found by walking up from the
//! current directory) or from a custom path.
//!
//! ## Features
//! - Load from the nearest `.env` above `std::env::current_dir()` (`dotenv()`)
//! - Load from an explicit path (`dotenv_from(path)`)
//! - Choose whether to overwrite existing variables (`*_override()` variants)
//! - Simple parser: `KEY=VALUE`, comments with `#`, optional quotes, and a
//! small set of escape sequences (e.g., `\n`, `\t`, `\\`, `\"`, `\'`).
//! - Supports optional `export KEY=...` prefix (ignored if present).
//! 
//! ## Examples
//! ```no_run
//! use stdt::utils::dotenv::{dotenv, dotenv_from_override};
//!
//! // Load the closest .env walking up from the current dir
//! // (does not overwrite already-set vars):
//! let count = dotenv().expect("failed to load .env");
//! println!("loaded {count} entries");
//!
//! // Load from a custom file path, *overwriting* existing variables:
//! let count = dotenv_from_override("config/dev.env").unwrap();
//! println!("overwrote {count} entries");
//! ```
//!

use std::collections::HashMap;
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};


/// Load the nearest `.env` by walking upward from `current_dir()`.
/// Returns the number of variables applied.
pub fn dotenv() -> Result<usize, Error> {
    let path = find_dotenv(None)?;
    dotenv_from_impl(&path, false)
}


/// Load the nearest `.env` by walking upward; **overwrite** existing vars.
/// Returns the number of variables applyied.
pub fn dotenv_override() -> Result<usize, Error> {
    let path = find_dotenv(None)?;
    dotenv_from_impl(&path, true)
}

/// Load variables from an explicit file path; do **not** overwrite
/// existing variables.
pub fn dotenv_from<P: AsRef<Path>>(path: P) -> Result<usize, Error> {
    dotenv_from_impl(path.as_ref(), false)
}


/// Load variables from an explicit file path; **overwrite** existing vars.
pub fn dotenv_from_override<P: AsRef<Path>>(path: P) -> Result<usize, Error> {
    dotenv_from_impl(path.as_ref(), true)
}


fn dotenv_from_impl(path: &Path, overwrite: bool) -> Result<usize, Error> {
    let file = File::open(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let reader = BufReader::new(file);
    let parsed = parse_reader(reader)?;
    let applied = apply_to_env(parsed, overwrite);
    Ok(applied)
}


/// Error type for this crate.
#[derive(Debug)]
pub enum Error {
    /// IO error while opening/reading a file.
    Io { path: PathBuf, source: io::Error },
    /// A syntactic error at a specific line number (1-based).
    Parse { path: Option<PathBuf>, line: usize, msg: String },
    /// `.env` file not found while walking up from a directory.
    NotFound { start_dir: PathBuf },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "io error reading {}: {}", path.display(), source),
            
            Error::Parse { path, line, msg } => {

                match path {
                    Some(p) => write!(f, "parse error in {} at line {}: {}", p.display(), line, msg),
                    None => write!(f, "parse error at line {}: {}", line, msg),
                }
            }

            Error::NotFound { start_dir } => write!(f, ".env not found (start: {})", start_dir.display()),
        }
    }
}


impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn find_dotenv(start: Option<PathBuf>) -> Result<PathBuf, Error> {
    let mut dir = start.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    loop {
        let candidate = dir.join(".env");
        
        if candidate.is_file() {
            return Ok(candidate);
        }

        if !dir.pop() {
            return Err(Error::NotFound { start_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")) });
        }
    }
}

fn parse_reader<R: BufRead>(mut reader: R) -> Result<HashMap<String, String>, Error> {
    let mut buf = String::new();
    let mut map = HashMap::new();
    let mut line_no = 0usize;

    
    loop {
        buf.clear();
        let bytes = reader.read_line(&mut buf).map_err(|e| Error::Io { path: PathBuf::from("<reader>"), source: e })?;

        if bytes == 0 { break; }
        line_no += 1;

        let trimmed = buf.trim_end_matches(['\n', '\r']);
        
        if trimmed.trim().is_empty() { continue; }
        match parse_line(trimmed) {
            Line::Comment | Line::Blank => {}
            Line::Pair { key, value } => { map.insert(key, value); }
            Line::Err(msg) => return Err(Error::Parse { path: None, line: line_no, msg }),
        }
    }   

    Ok(map)
}

#[derive(Debug, PartialEq, Eq)]
enum Line {
    Comment,
    Blank,
    Pair { key: String, value: String },
    Err(String),
}

fn parse_line(s: &str) -> Line {
    let s = s.trim();
    if s.is_empty() { return Line::Blank; }
    if s.starts_with('#') { return Line::Comment; }

    let s = s.strip_prefix("export ")
             .or_else(|| s.strip_prefix("export\t"))
             .map(|t| t)
             .unwrap_or(s);

    let mut in_single = false;
    let mut in_double = false;
    let mut iter = s.char_indices().peekable();
    let mut key = String::new();
    let mut val = String::new();
    let mut saw_eq = false;

    while let Some((i, ch)) = iter.next() {
        match ch {
            '=' if !in_single && !in_double => {
                key = s[..i].trim().to_string();
                val = s[i+1..].to_string();
                saw_eq = true;
                break;
            }
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => {
                in_double = !in_double;
            }
            '\\' if in_double => {
                let _ = iter.next();
            }
            _ => {}
        }
    }

    if !saw_eq { return Line::Err("missing '='".into()); }
    if !is_valid_key(&key) { return Line::Err("invalid key".into()); }

    // Rimuovi commento inline solo se il valore NON è quotato e il '#' è "vero commento"
    let val_trimmed = val.trim_start();
    let val = if !(val_trimmed.starts_with('"') || val_trimmed.starts_with('\'')) {
        strip_inline_comment_if_unquoted(&val)
    } else {
        val
    };

    let value = unquote_and_unescape(val.trim());
    match value {
        Ok(v) => Line::Pair { key, value: v },
        Err(msg) => Line::Err(msg),
    }
}

fn is_valid_key(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() { 
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}, 
        _ => return false 
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}


fn unquote_and_unescape(raw: &str) -> Result<String, String> {
    let raw = raw.trim();

    if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
        return Ok(raw[1..raw.len() - 1].to_string());
    }

    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        let s = &raw[1..raw.len() - 1];
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('0') => out.push('\0'),
                    Some('"') => out.push('"'),
                    Some('\'') => out.push('\''),
                    Some('\\') => out.push('\\'),
                    Some(other) => { out.push('\\'); out.push(other); } 
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        return Ok(out);
    }

    Ok(raw.to_string())
}

fn strip_inline_comment_if_unquoted(val: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let mut prev_is_space_or_start = true;
    for (i, ch) in val.char_indices() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double && prev_is_space_or_start => {
                return val[..i].to_string();
            }
            _ => {}
        }
        prev_is_space_or_start = ch.is_whitespace();
    }
    val.to_string()
}

fn apply_to_env(map: HashMap<String, String>, overwrite: bool) -> usize {
    let mut applied = 0usize;
    for (k, v) in map.into_iter() {
        let should_set = overwrite || env::var_os(&k).is_none();

        if should_set {
            // SAFETY: This function is only called during process initialization
            // before any threads are spawned. Mutating the global environment
            // at this stage is free of data races and thus considered safe.
            unsafe{env::set_var(&k, &v);}
            applied += 1;
        }
    }

    applied
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }
    
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let mut base = env::temp_dir();
            let unique_name = format!("dotenv-mini-test-{}", std::process::id());
            base.push(unique_name);
            let _ = fs::remove_dir_all(&base);
            fs::create_dir_all(&base).expect("Failed to create temp dir");
            TempDir(base)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    
    #[test]
    fn parse_basic_kv() {
        assert_eq!(parse_line("FOO=bar"), Line::Pair { key: "FOO".into(), value: "bar".into() });
        assert_eq!(parse_line("export FOO=bar"), Line::Pair { key: "FOO".into(), value: "bar".into() });
        assert!(matches!(parse_line("_A=1"), Line::Pair{..}));
        assert!(matches!(parse_line("9A=1"), Line::Err(_)));
    }

    #[test]
    fn comments_and_blank_lines() {
        assert!(matches!(parse_line("# hello"), Line::Comment));
        assert!(matches!(parse_line(" # spaced"), Line::Comment));
        assert!(matches!(parse_line(" "), Line::Blank));
        assert_eq!(parse_line("FOO=bar # trailing"), Line::Pair { key: "FOO".into(), value: "bar".into() });
    }

    #[test]
    fn quotes_and_escapes() {
        assert_eq!(parse_line("X=\\n").unwrap_pair().1, "\\n");
        assert_eq!(parse_line("X='a b' ").unwrap_pair().1, "a b");
        assert_eq!(parse_line("X='line\\nfeed'").unwrap_pair().1, "line\\nfeed");
        assert_eq!(parse_line("X='quote: \\\''").unwrap_pair().1, "quote: \\\'");
        assert_eq!(parse_line("X=\"a b\" ").unwrap_pair().1, "a b");
        assert_eq!(parse_line("X=\"line\\nfeed\"").unwrap_pair().1, "line\nfeed");
        assert_eq!(parse_line("X=\"quote: \\\"\"").unwrap_pair().1, "quote: \"");
    }

    #[test]
    fn apply_respects_overwrite() {
        let _lock = lock(); // <--- ACQUISISCE IL LOCK
        let dir = TempDir::new(); // <--- USA LA STRUCT RAII
        let file = dir.path().join(".env");
        fs::write(&file, "A=1\nB=2\n").unwrap();

        // Salva e ripristina la CWD per isolare il test
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();
       
        // SAFETY: Removing an environment variable is performed only in 
        // controlled, single-threaded contexts (tests or initialization). 
        // No concurrent access to the environment occurs, so this call 
        // does not introduce undefined behavior. 
        unsafe {
            env::remove_var("A");
            env::set_var("B", "pre");
        }

        dotenv_from(&file).unwrap();
        assert_eq!(env::var("A").unwrap(), "1");
        assert_eq!(env::var("B").unwrap(), "pre");

        dotenv_from_override(&file).unwrap();
        assert_eq!(env::var("B").unwrap(), "2");
        
        // Ripristina la directory originale
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn walk_up_finds_nearest_dotenv() {
        let _lock = lock(); // <--- ACQUISISCE IL LOCK
        let root = TempDir::new(); // <--- USA LA STRUCT RAII
        let sub = root.path().join("a/b/c");
        fs::create_dir_all(&sub).unwrap();
        fs::write(root.path().join(".env"), "ROOT=1\n").unwrap();
        fs::write(root.path().join("a/.env"), "A=2\n").unwrap();
    
        // SAFETY: Removing an environment variable is performed only in 
        // controlled, single-threaded contexts (tests or initialization). 
        // No concurrent access to the environment occurs, so this call 
        // does not introduce undefined behavior. 
        unsafe {
            env::remove_var("A");
            env::remove_var("ROOT");
        }

        let cwd = env::current_dir().unwrap();
        env::set_current_dir(&sub).unwrap();
        
        let count = dotenv().unwrap();
        
        env::set_current_dir(cwd).unwrap();

        // Should pick a/.env (nearest), applying 1 variable
        assert_eq!(count, 1);
        assert_eq!(env::var("A").unwrap(), "2");
        assert!(env::var("ROOT").is_err());
    }

    // ---- Helpers ----
    trait UnwrapPair { fn unwrap_pair(self) -> (String, String); }
    impl UnwrapPair for Line {
        fn unwrap_pair(self) -> (String, String) {
            match self {
                Line::Pair{key, value} => (key, value),
                other => panic!("expected Pair, got {:?}", other)
            }
        }
    }
}

