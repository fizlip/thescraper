# thescraper

A webscraper used to get all laws from the swedish government [website](https://www.riksdagen.se/sv/sok/?doktyp=sfs&dokstat=g%C3%A4llande+sfs&p=1).
It uses the scraper and reqwest libraries to read and extract specific tags
from the html pages of sites.

It is used to create a corpus of law text that can be used to create AI agents
with specific knowledge of the swedish justice system.

## How it works
The government website is structured as follows:

A paginated list of laws (there are 268 pages)
```
https://www.riksdagen.se/sv/sok/?doktyp=sfs&dokstat=g%C3%A4llande+sfs&p=PAGE
```
each page has 20 links that are extracted. Each page in the link contains 
another link to the official source on the website of the government office.
```
http://rkrattsbaser.gov.se/sfsr?bet=LAW_ID
```
This site contains the raw text in a div that has the class 'body-text'.

In summary the scraper gets 20 links per page then extract a link for each of
those and then gets the raw text.

```
page link -> law page link -> raw text from official source
```
We do this for all 268 pages on the website, resulting in a structured dataset
of all swedish laws. In total we will be making 268 * 20 * 20 = 107200 requests.
With a normal connection it takes about 1 day to get all laws.
## Usage

```
git clone git@github.com:fizlip/thescraper.git
cd thesrcaper
cargo test
cargo run
```
