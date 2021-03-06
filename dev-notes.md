## Tools

* Wikipedia SQL dumps: https://dumps.wikimedia.org/enwiki/
* Online DB: https://quarry.wmflabs.org/
  - (use `WHERE id > 1000000 AND id < 2000000` to prevent expensive queries from timing out)
* Show Wiki page by ID: `https://en.wikipedia.org/w/index.php?redirect=no&curid=<page_id>`

## Related Wikipedia articles

* https://en.wikipedia.org/wiki/Wikipedia:Most-referenced_articles
* https://en.wikipedia.org/wiki/Special:BrokenRedirects
* https://en.wikipedia.org/wiki/Special:DoubleRedirects
* https://en.wikipedia.org/wiki/Category:Redirects_to_special_pages

## Possible features

* Option to treat redirects as regular pages, not combining their counts with their target page.
* Option to count links for 1 or more *specific* pages
* More output formats:
  - XML
  - JSON
  - HTML (table)

