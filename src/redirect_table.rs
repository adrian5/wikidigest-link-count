/*
Process SQL dumps for the MediaWiki “redirect” table.
*/
use crate::{
    buffer_queue::BufferQueue,
    chunked_reader::ChunkedReader,
    progress_display::ProgressDisplay,
    util::{self, PageId, PageNs, PageTitle}
};

use ahash::AHashMap;
use anyhow::{Context, Result};
use regex::Regex;

use std::io::Read;
use std::sync::Mutex;

pub fn map_redirects<T>(source: T, pages: AHashMap<(PageNs, PageId), PageTitle>,
    namespaces: &[PageNs], buffer_size: usize)
    -> Result<AHashMap<(PageNs, PageTitle), PageTitle>> where T: Read + Send {
    let redirects: Mutex<AHashMap<(PageNs, PageTitle), PageTitle>> = Mutex::new(AHashMap::new());

    let mut source = ChunkedReader::new(source);
    let buffers = BufferQueue::new(num_cpus::get() + 1, buffer_size);
    let regex = build_redirect_regex(namespaces)?;

    let mut progress = ProgressDisplay::new(buffer_size);

    rayon::scope_fifo(|s| -> Result<()> {
        let pages = &pages;
        let redirects = &redirects;
        let regex = &regex;

        loop {
            eprint!("\r2/5 Extracting ‘redirect’ table data and mapping relations \
                ({:.1} GiB processed)", progress.next());
            let buffer = buffers.pop();
            let was_final_read = !source.read_into(&mut buffer.borrow(), buffer_size)?;

            s.spawn_fifo(move |_| {
                let mut new_redirects: Vec<((PageNs, PageTitle), PageTitle)> = Vec::new();
                for cap in regex.captures_iter(&buffer.borrow()) {
                    let source_id = PageId(cap[1].parse::<u32>().unwrap());
                    let source_ns = PageNs(cap[2].parse::<u32>().unwrap());
                    let target_title = &cap[3];

                    if let Some(source_title) = pages.get(&(source_ns, source_id)) {
                        new_redirects.push(((source_ns, source_title.clone()),
                            PageTitle(target_title.to_string())));
                    }
                }

                buffer.release();

                let mut redirects = redirects.lock().unwrap();
                redirects.extend(new_redirects.into_iter());
            });

            if was_final_read {
                break;
            }
        }

        eprintln!(" Done.");
        Ok(())
    })?;

    Ok(redirects.into_inner().unwrap())
}

/*
Regex Pattern: \((\d+),({}),'((?:[^']|\\'){1,255}?)','','(?:[^']|\\'){1, 255}?'\)

(\d+) : match and capture the ‘rd_from’ field.

,({}), : match and capture ‘rd_namespace’ on any of the given numbers (e.g. 0|5|7) passed via the
function parameter.

'((?:[^']|\\'){1,255}?)' : match and capture as ‘rd_title’, any UTF-8 sequence of up to 255 bytes
that does not contain ' except if escaped \'. Strictly speaking, the 255 byte limits is not needed,
but it offers some protection against fauly matches in the case of erroneous data.

,'', : match empty ‘rd_interwiki’ field, as we're not interested in links to external targets

'(?:[^']|\\'){0,255}?' : matches the ‘rd_fragment’ field similarly to ‘rd_title’, but allowing it
to be empty.
*/
fn build_redirect_regex(namespaces: &[PageNs]) -> Result<Regex> {
    let ns_str = util::namespaces_to_string(namespaces);

    let pattern = format!(r"\((\d+),({}),'((?:[^']|\\'){{1,255}}?)','','(?:[^']|\\'){{0,255}}?'\)",
        ns_str);
    Regex::new(&pattern).context("Building redirect pattern")
}
