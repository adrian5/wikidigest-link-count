/*
Shared utility types and functions
*/
use crate::link_count::LinkCount;

use ahash::AHashMap;
use anyhow::Result;

use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq, PartialOrd)]
pub struct PageId(pub u32);

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq, PartialOrd)]
pub struct PageNs(pub u32);

impl fmt::Display for PageNs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Default, Eq, Hash, PartialEq, PartialOrd)]
pub struct PageTitle(pub String);

impl fmt::Display for PageTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy)]
pub enum ExportFormat {
    PlainText
}

pub fn is_probably_gzip(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if ext == "gz" || ext == "gzip" {
            return true;
        }
    }
    false
}

pub fn build_output_filename(path: &Path, export_format: ExportFormat) -> PathBuf {
    use ExportFormat::*;
    let mut filename = path.to_path_buf();

    if path.extension().is_some() { // User supplied extension
        return filename
    }

    match export_format {
        PlainText => { filename.set_extension("txt"); }
    }

    filename
}

pub fn namespaces_to_string(namespaces: &[PageNs]) -> String {
    use std::fmt::Write;

    let mut ns_str = String::new();
    if namespaces.len() == 1 {
        ns_str = format!("{}", namespaces[0].0);
    } else {
        // In the case of multi-digit numbers, matching in
        // big-to-small order allows more efficient regex logic
        let mut namespaces = namespaces.to_vec();
        namespaces.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        for ns in namespaces {
            write!(&mut ns_str, "{}|", ns.0).unwrap();
        }
        ns_str.pop(); // Trailing '|'
    }

    ns_str
}

fn underscores_to_spaces(mut s: String) -> String {
    unsafe {
        for c in s.as_bytes_mut() {
            if *c == b'_' {
                *c = b' ';
            }
        }
    }
    s
}

pub fn sort_pagelinks(pagelinks: AHashMap<(PageNs, PageTitle), LinkCount>, cutoff: u32)
    -> Vec<((PageNs, PageTitle), LinkCount)> {
    let mut output: Vec<((PageNs, PageTitle), LinkCount)> = pagelinks.into_iter()
        .filter(|pl| pl.1.total() >= cutoff)
        .collect();

    output.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    output
}

pub fn export_to_file(pages: Vec<((PageNs, PageTitle), LinkCount)>, mut file: File,
    format: ExportFormat) -> Result<()> {
    use ExportFormat::*;

    match format {
        PlainText => write_plaintext(&mut file, pages)?,
    }

    Ok(())
}

fn write_plaintext(file: &mut File, pages: Vec<((PageNs, PageTitle), LinkCount)>) -> Result<()> {
    writeln!(file, "page title [namespace]  →  links-total (direct + indirect)\n")?;
    for p in pages {
        let title = underscores_to_spaces(((p.0).1).0);
        writeln!(file, "{} [{}]  →  {} ({} + {})",
            title, (p.0).0, p.1.total(), p.1.direct, p.1.indirect)?;
    }
    Ok(())
}