/*
Process SQL dumps for the MediaWiki “pagelinks” table.
*/
use crate::{
    buffer_queue::BufferQueue,
    chunked_reader::ChunkedReader,
    link_count::LinkCount,
    progress_display::ProgressDisplay,
    util::{self, PageNs, PageTitle},
};

use ahash::AHashMap;
use anyhow::{Context, Result};
use regex::Regex;

use std::io::Read;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct Counter {
    pub direct: u32,
    pub indirect: u32,
}

pub fn count_links<T>(
    source: T,
    redirects: AHashMap<(PageNs, PageTitle), PageTitle>,
    namespaces: (&[PageNs], &[PageNs]),
    buffer_size: usize,
) -> Result<AHashMap<(PageNs, PageTitle), LinkCount>>
where
    T: Read + Send,
{
    let pagelinks: Mutex<AHashMap<(PageNs, PageTitle), LinkCount>> = Mutex::new(AHashMap::new());

    let mut source = ChunkedReader::new(source);
    let buffers = BufferQueue::new(num_cpus::get() + 1, buffer_size);
    let regex = build_pagelinks_regex(namespaces.0, namespaces.1)?;

    let mut progress = ProgressDisplay::new(buffer_size);

    rayon::scope_fifo(|s| -> Result<()> {
        let pagelinks = &pagelinks;
        let redirects = &redirects;
        let regex = &regex;

        loop {
            eprint!(
                "\r3/5 Extracting ‘pagelinks’ table data and counting links \
                    ({:.1} GiB processed)",
                progress.next()
            );
            let buffer = buffers.pop();
            let was_final_read = !source.read_into(&mut buffer.borrow(), buffer_size)?;

            s.spawn_fifo(move |_| {
                let mut new_pagelinks = AHashMap::<(PageNs, PageTitle), LinkCount>::new();

                for cap in regex.captures_iter(&buffer.borrow()) {
                    let ns = PageNs(cap[1].parse::<u32>().unwrap());
                    let title = &cap[2];

                    if let Some(re_title) = rd_query(&redirects, ns, title) {
                        if let Some(link_count) = pl_query(&mut new_pagelinks, ns, &re_title.0) {
                            link_count.indirect += 1;
                        } else {
                            new_pagelinks.insert((ns, re_title.clone()), LinkCount::new(0, 1));
                        }
                    } else { // Title is not a redirect
                        if let Some(link_count) = pl_query(&mut new_pagelinks, ns, title) {
                            link_count.direct += 1;
                        } else {
                            new_pagelinks
                                .insert((ns, PageTitle(title.to_string())), LinkCount::new(1, 0));
                        }
                    }
                }

                buffer.release();

                let mut pagelinks = pagelinks.lock().unwrap();
                for (page, new_counter) in new_pagelinks {
                    if let Some(counter) = pagelinks.get_mut(&page) {
                        *counter += new_counter;
                    } else {
                        pagelinks.insert(page, new_counter);
                    }
                }
            });

            if was_final_read {
                break;
            }
        }

        eprintln!(" Done.");
        Ok(())
    })?;

    Ok(pagelinks.into_inner().unwrap())
}

/*
Regex Pattern: \(\d+,({}),'((?:[^']|\\'){1,255}?)',(?:{})\)

\d+ : match the ‘pl_from’ field.

,({}), : match and capture ‘pl_namespace’ on any of the given numbers (e.g. 0|5|7) passed via the
second namespaces function parameter.

'((?:[^']|\\'){1,255}?)' : match and capture as ‘pl_title’, any UTF-8 sequence of up to 255 bytes
that does not contain ' except if escaped as \'. Strictly speaking, the 255 byte limit is not
needed, but it offers some protection against erroneous (long) matches in the case of faulty data.

,(?:{}), : match the ‘pl_from_namespace’ on any of the numbers passed via the first namespaces
function parameter.
*/
fn build_pagelinks_regex(namespaces_from: &[PageNs], namespaces_to: &[PageNs]) -> Result<Regex> {
    let ns_from_str = util::namespaces_to_string(namespaces_from);
    let ns_to_str = util::namespaces_to_string(namespaces_to);

    let pattern = format!(
        r"\(\d+,({}),'((?:[^']|\\'){{1,255}}?)',(?:{})\)",
        ns_to_str, ns_from_str
    );
    Regex::new(&pattern).context("Building pagelinks regex")
}

/*
These helper functions work around a current limitation in Rust's stdlib HashMap interface, wherein
the lifetime of a query parameter is required to encompass that of the HashMap. Since we work with
temporary &str from the regex iterator, they would require cloning into String for each query.
*/
#[inline]
fn rd_query<'a>(
    rd: &'a AHashMap<(PageNs, PageTitle), PageTitle>,
    ns: PageNs,
    title: &str,
) -> Option<&'a PageTitle> {
    unsafe {
        // Satisfy the &(PageNs, PageTitle) interface without reallocations
        let key = (
            ns,
            PageTitle(String::from_raw_parts(
                title.as_ptr() as *mut u8,
                title.len(),
                title.len(),
            )),
        );
        let key = std::mem::ManuallyDrop::new(key);
        rd.get(&*key)
    }
}

#[inline]
fn pl_query<'a>(
    pl: &'a mut AHashMap<(PageNs, PageTitle), LinkCount>,
    ns: PageNs,
    title: &str,
) -> Option<&'a mut LinkCount> {
    unsafe {
        // As function above
        let key = (
            ns,
            PageTitle(String::from_raw_parts(
                title.as_ptr() as *mut u8,
                title.len(),
                title.len(),
            )),
        );
        let key = std::mem::ManuallyDrop::new(key);
        pl.get_mut(&*key)
    }
}
