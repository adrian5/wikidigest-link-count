mod buffer_queue;
mod chunked_reader;
mod cli;
mod link_count;
mod page_table;
mod pagelinks_table;
mod progress_display;
mod redirect_table;
mod util;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;

use std::fs::File;

const MIBI: usize = 1024 * 1024;

fn main() -> Result<()> {
    let cli = cli::init_cli_app()?;
    let buf_size = cli.buf_size_mib * MIBI;

    // Ensure output is writable before starting any processing
    let output_file = {
        let path = util::build_output_filename(&cli.output_file, cli.export_format);
        let file = File::create(&path)
            .with_context(|| format!("Failed to create output file ‘{}’", &path.display()))?;
        (file, path)
    };

    // Process page-table data
    let pages = {
        let f = File::open(&cli.page_file)
            .with_context(|| format!("Failed to open page file ‘{}’", &cli.page_file.display()))?;
        if util::is_probably_gzip(&cli.page_file) {
            page_table::collect_pages(GzDecoder::new(f), &cli.namespaces_to, buf_size)
        } else {
            page_table::collect_pages(f, &cli.namespaces_to, buf_size)
        }
    }?;

    // Process redirect-table data
    let redirects = {
        let f = File::open(&cli.redirect_file).with_context(|| {
            format!(
                "Failed to open redirect file ‘{}’",
                &cli.redirect_file.display()
            )
        })?;
        if util::is_probably_gzip(&cli.redirect_file) {
            redirect_table::map_redirects(GzDecoder::new(f), pages, &cli.namespaces_to, buf_size)
        } else {
            redirect_table::map_redirects(f, pages, &cli.namespaces_to, buf_size)
        }
    }?;

    // Process pagelinks-table data
    let pagelinks = {
        let f = File::open(&cli.pagelinks_file).with_context(|| {
            format!(
                "Failed to open pagelinks file ‘{}’",
                &cli.pagelinks_file.display()
            )
        })?;
        if util::is_probably_gzip(&cli.pagelinks_file) {
            pagelinks_table::count_links(
                GzDecoder::new(f),
                redirects,
                (&cli.namespaces_from, &cli.namespaces_to),
                buf_size,
            )
        } else {
            pagelinks_table::count_links(
                f,
                redirects,
                (&cli.namespaces_from, &cli.namespaces_to),
                buf_size,
            )
        }
    }?;

    // Reduce dataset to pages with link count above threshold, and sort in descending order
    eprint!("4/5 Sorting pages (...)");
    let pagelinks = util::sort_pagelinks(pagelinks, cli.cutoff_threshold);
    eprintln!(" Done.");

    // Write output
    eprint!("5/5 Writing results to {} (...)", output_file.1.display());
    util::export_to_file(pagelinks, output_file.0, cli.export_format)?;
    eprintln!(" Done.");

    Ok(())
}
