/*
Process SQL dumps for the MediaWiki “page” table.
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

pub fn collect_pages<T>(source: T, namespaces: &[PageNs], buffer_size: usize)
    -> Result<AHashMap<(PageNs, PageId), PageTitle>> where T: Read + Send {
    let pages: Mutex<AHashMap<(PageNs, PageId), PageTitle>> = Mutex::new(AHashMap::new());

    let mut source = ChunkedReader::new(source);
    let buffers = BufferQueue::new(num_cpus::get() + 1, buffer_size);
    let regex = build_page_regex(namespaces)?;

    let mut progress = ProgressDisplay::new(buffer_size);

    rayon::scope_fifo(|s| -> Result<()> {
        let pages = &pages;
        let regex = &regex;

        loop {
            eprint!("\r1/5 Extracting ‘page’ table data ({:.1} GiB processed)", progress.next());
            let buffer = buffers.pop();
            let was_final_read = !source.read_into(&mut buffer.borrow(), buffer_size)?;

            s.spawn_fifo(move |_| {
                let mut new_pages = Vec::new();
                for cap in regex.captures_iter(&buffer.borrow()) {
                    let id = PageId(cap[1].parse::<u32>().unwrap());
                    let ns = PageNs(cap[2].parse::<u32>().unwrap());
                    let title = PageTitle(cap[3].to_string());

                    new_pages.push(((ns, id), title));
                }

                buffer.release();

                let mut pages = pages.lock().unwrap();
                pages.extend(new_pages.into_iter());
            });

            if was_final_read {
                break;
            }
        }

        eprintln!(" Done.");
        Ok(())
    })?;

    Ok(pages.into_inner().unwrap())
}

/*
Regex Pattern: \((\d+),({}),'((?:[^']|\\'){1,255}?)','[a-z:=]*?',1,

(\d+) : match and capture the ‘page_id’ field.

,({}), : match and capture ‘page_namespace’ on any of the given numbers (e.g. 0|5|7) passed via the
function parameter.

'((?:[^']|\\'){1,255}?)' : match and capture as ‘page_title’, any UTF-8 sequence of up to 255 bytes
that does not contain ' except if escaped \'. Strictly speaking, the 255 byte limits is not needed,
but it offers some protection against fauly matches in the case of erroneous data.

'[a-z,:=]*?' : matches the ‘page_restrictions’ field. This field is deprecated, but matched here
for the sake of completeness. Cost seems negligible.

,1, : matches on the ‘page_is_redirect’ field being 1 (true), which we always want.
*/
fn build_page_regex(namespaces: &[PageNs]) -> Result<Regex> {
    let ns_str = util::namespaces_to_string(namespaces);

    let pattern = format!(r"\((\d+),({}),'((?:[^']|\\'){{1,255}}?)','[a-z,:=]*?',1,", ns_str);
    Regex::new(&pattern).context("Building page regex")
}

