/*
Parsing CLI arguments
*/
use crate::util::{ExportFormat, PageNs};

use anyhow::Result;
use clap::{App, Arg};

use std::convert::TryFrom;
use std::path::PathBuf;
use std::str::FromStr;

pub struct CliParams {
    pub page_file: PathBuf,
    pub redirect_file: PathBuf,
    pub pagelinks_file: PathBuf,
    pub output_file: PathBuf,
    pub namespaces_from: Vec<PageNs>,
    pub namespaces_to: Vec<PageNs>,
    pub buf_size_mib: usize,
    pub cutoff_threshold: u32,
    pub export_format: ExportFormat,
}

pub fn init_cli_app() -> Result<CliParams> {
    let matches = App::new("wikidigest-link-count")
        .version("0.1")
        .author("github.com/adrian5")
        .about("Find the most linked-to pages in MediaWiki databases")
        .after_help(
            "This program requires three files as input data:\n\n\
            1. The page-table SQL dump (…page.sql.gz)\n\
            2. The redirect-table SQL dump (…redirect.sql.gz)\n\
            3. The pagelinks-table SQL dump (…pagelinks.sql.gz)\n\n\
            For the English Wikipedia, you can get these at https://dumps.wikimedia.org/enwiki/",
        )
        // Page file
        .arg(
            Arg::with_name("file-page")
                .short("p")
                .long("page-file")
                .value_name("PATH")
                .help("Path to ‘…page.sql(.gz)’")
                .takes_value(true)
                .required(true),
        )
        // Redirect file
        .arg(
            Arg::with_name("file-redirect")
                .short("r")
                .long("redirect-file")
                .value_name("PATH")
                .help("Path to ‘…redirect.sql(.gz)’")
                .takes_value(true)
                .required(true),
        )
        // Pagelinks file
        .arg(
            Arg::with_name("file-pagelinks")
                .short("l")
                .long("pagelinks-file")
                .value_name("PATH")
                .help("Path to ‘…pagelinks.sql(.gz)’")
                .takes_value(true)
                .required(true),
        )
        // Output file
        .arg(
            Arg::with_name("file-output")
                .short("o")
                .long("output-file")
                .value_name("PATH")
                .help("Path to write results to")
                .default_value("./results")
                .takes_value(true),
        )
        // Namespaces (From)
        .arg(
            Arg::with_name("namespaces-from")
                .short("f")
                .long("from-namespaces")
                .value_name("ns,ns,…")
                .help("Namespace(s) of pages from which links may originate")
                .default_value("0")
                .takes_value(true)
                .use_delimiter(true),
        )
        // Namespaces (To)
        .arg(
            Arg::with_name("namespaces-to")
                .short("t")
                .long("to-namespaces")
                .value_name("ns,ns,…")
                .help("Namespace(s) of pages to which links may lead")
                .default_value("0")
                .takes_value(true)
                .use_delimiter(true),
        )
        // Buffer size
        .arg(
            Arg::with_name("buf-size")
                .short("b")
                .long("bufsize")
                .value_name("MiB")
                .help("Buffer size per thread")
                .default_value("32")
                .takes_value(true)
                .validator(|bs| match bs.parse::<u32>() {
                    Err(_) => Err("must be a positive number".to_string()),
                    Ok(value) => {
                        if value > 8 && value < 1024 {
                            Ok(())
                        } else {
                            Err("must be between 8 and 1024".to_string())
                        }
                    }
                }),
        )
        // Cutoff threshold
        .arg(
            Arg::with_name("cutoff-threshold")
                .short("c")
                .long("cutoff")
                .value_name("THRESHOLD")
                .help("Output only pages with link-count above threshold")
                .default_value("25000")
                .takes_value(true)
                .validator(|t| {
                    t.parse::<u32>()
                        .map(|_| ())
                        .map_err(|_| "must be a positive number".to_string())
                }),
        )
        // Export format
        .arg(
            Arg::with_name("export-format")
                .short("e")
                .long("export-as")
                .value_name("FORMAT")
                .help("Format to output results as")
                .long_help("Supported formats are: text (plain), wikitext, markdown (gfm)")
                .default_value("text")
                .takes_value(true)
                .validator(|f| {
                    ExportFormat::try_from(f.as_str())
                        .map(|_| ())
                        .map_err(|e| e)
                }),
        )
        .get_matches();

    // Conversion
    let page_file = PathBuf::from_str(matches.value_of("file-page").unwrap())?;
    let redirect_file = PathBuf::from_str(matches.value_of("file-redirect").unwrap())?;
    let pagelinks_file = PathBuf::from_str(matches.value_of("file-pagelinks").unwrap())?;
    let output_file = PathBuf::from_str(matches.value_of("file-output").unwrap())?;

    let namespaces_from = matches
        .values_of("namespaces-from")
        .unwrap()
        .map(|ns| PageNs(ns.parse::<u32>().unwrap()))
        .collect::<Vec<PageNs>>();
    let namespaces_to = matches
        .values_of("namespaces-to")
        .unwrap()
        .map(|ns| PageNs(ns.parse::<u32>().unwrap()))
        .collect::<Vec<PageNs>>();

    let buf_size_mib = matches.value_of("buf-size").unwrap().parse::<usize>()?;
    let cutoff_threshold = matches
        .value_of("cutoff-threshold")
        .unwrap()
        .parse::<u32>()?;
    let export_format = ExportFormat::try_from(matches.value_of("export-format").unwrap()).unwrap();

    let cli_params = CliParams {
        page_file,
        redirect_file,
        pagelinks_file,
        output_file,
        buf_size_mib,
        cutoff_threshold,
        namespaces_from,
        namespaces_to,
        export_format,
    };

    Ok(cli_params)
}
