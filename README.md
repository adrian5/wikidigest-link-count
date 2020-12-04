# wikidigest-link-count

Small tool to find the most linked-to pages within a MediaWiki database.

## Installation

Download the executable for your system from the [releases](https://github.com/adrian5/wikidigest-link-count/releases/latest) page.
See [requirements](#requirements) below.

You may have to `chmod u+x` the executable on Linux/MacOS.

### Building from source

1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Clone or download this git repository to a directory on your machine
3. From within that directory, run `cargo build --release`
4. Find the `wikidigest-link-count` executable under `./target/release/`

## Requirements

The program requires three input files to operate:

1. A [page table](https://www.mediawiki.org/wiki/Manual:Page_table) dump¹
2. A [redirect table](https://www.mediawiki.org/wiki/Manual:Redirect_table) dump¹
3. A [pagelinks table](https://www.mediawiki.org/wiki/Manual:Pagelinks_table) dump¹

¹Plain or GZIP compressed

For the English Wikipedia, you can get these at <https://dumps.wikimedia.org/enwiki/> as:

* enwiki-yyyymmdd-page.sql.gz
* enwiki-yyyymmdd-redirect.sql.gz
* enwiki-yyyymmdd-pagelinks.sql.gz

### Hardware

For very large Wikis like the English Wikipedia, I recommend more than 4GB RAM to be on the safe
side. Processing time depends strongly on the amount of input data and hardware used, but expect
no more than 30 minutes.

## Usage

See `wikidigest-link-count --help` for options.

### Examples

English Wikipedia, namespaces 0 → 0 (default):

```
wikidigest-link-count -p enwiki-20200501-page.sql.gz -r enwiki-20200501-redirect.sql.gz -l enwiki-20200501-pagelinks.sql.gz
```

Links from pages in namespaces 0 and 4, leading to pages in namespaces 5, 8 and 16:

```
wikidigest-link-count -p page.sql -r redirect.sql -l pagelinks.sql -o ./custom-results -f 0,4 -t 5,8,16
```

Custom (128 MiB) buffer size and a link-count cutoff of 185K, below which pages are discarded:

```
wikidigest-link-count -p page.sql.gz -r redirect.sql.gz -l pagelinks.sql.gz -o /tmp/185k-or-more -b 128 -c 185000
```

Export as different format ([WikiText](https://en.wikipedia.org/wiki/Help:Wikitext) table):

```
wikidigest-link-count -p page.sql -r redirect.sql -l pagelinks.sql -e wikitext
```

## Results

Results are written to an output file, by default as Plaintext to `./results.txt`.

Below results for the English Wikipedia, Apr 2020 – pages with 200K or more incoming links
within the main (0) namespace:

| Page                                            | Links total | Direct    | via Redirect |
| :---------------------------------------------- | ----------: | --------: | -----------: |
| International Standard Book Number              | 1,145,265   | 150,863   | 994,402      |
| Geographic coordinate system                    | 1,119,214   | 1,116,647 | 2,567        |
| Virtual International Authority File            | 685,199     | 680,992   | 4,207        |
| WorldCat                                        | 644,016     | 7,715     | 636,301      |
| Library of Congress Control Number              | 536,790     | 522,775   | 14,015       |
| United States                                   | 484,831     | 466,785   | 18,046       |
| International Standard Name Identifier          | 444,828     | 444,814   | 14           |
| Wikidata                                        | 428,618     | 428,617   | 1            |
| Diacritic                                       | 411,701     | 1,162     | 410,539      |
| Time zone                                       | 409,222     | 409,026   | 196          |
| Taxonomy (biology)                              | 407,136     | 405,354   | 1,782        |
| Wayback Machine                                 | 395,845     | 395,656   | 189          |
| Digital object identifier                       | 361,935     | 73,326    | 288,609      |
| Global Biodiversity Information Facility        | 346,181     | 346,057   | 124          |
| Binomial nomenclature                           | 327,570     | 316,121   | 11,449       |
| Integrated Authority File                       | 323,249     | 318,607   | 4,642        |
| IMDb                                            | 321,316     | 310,843   | 10,473       |
| Animal                                          | 318,644     | 290,146   | 28,498       |
| List of sovereign states                        | 270,436     | 139,127   | 131,309      |
| Interim Register of Marine and Nonmarine Genera | 268,770     | 268,770   | 0            |
| Bibliothèque nationale de France                | 266,909     | 261,530   | 5,379        |
| Daylight saving time                            | 263,874     | 263,541   | 333          |
| France                                          | 236,307     | 235,797   | 510          |
| Encyclopedia of Life                            | 231,131     | 231,078   | 53           |
| United Kingdom                                  | 230,017     | 219,072   | 10,945       |
| Association football                            | 221,867     | 186,767   | 35,100       |
| Système universitaire de documentation          | 215,394     | 213,057   | 2,337        |
| Record label                                    | 214,541     | 213,508   | 1,033        |
| Race and ethnicity in the United States Census  | 210,066     | 2,809     | 207,257      |
| Arthropod                                       | 204,965     | 183,319   | 21,646       |
| Music genre                                     | 204,569     | 204,248   | 321          |
| INaturalist                                     | 203,702     | 203,702   | 0            |
| Germany                                         | 200,581     | 199,546   | 1,035        |

## Issues

If you have any problems, feel free to use the [issue tracker](https://github.com/adrian5/wikidigest-link-count/issues).
