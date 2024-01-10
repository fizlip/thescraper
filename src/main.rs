use std::error::Error;
use futures::executor::block_on;
use scraper::{Html, Selector};
use scraper::html::Select;
use std::iter::Enumerate;
use scraper::ElementRef;
use regex::Regex;
use std::fs;
use more_asserts::assert_le;

const PAGE_SIZE:usize = 20;
const LINK_TAG: &str = "a.sc-3189427c-0";
const RAW_TEXT_TAG: &str = "div.body-text";

/// main_page_links will retrieve all law links from www.riksdagen.se
/// @params i -- the page to retrieve
/// @params take -- the page size
/// @returns Vec<String> -- vector of links to all laws on the page
fn main_page_links(i: u64, take: usize) -> Vec<String>{
    // Get HTML 
    let url = format!("https://www.riksdagen.se/sv/sok/?doktyp=sfs&dokstat=g%C3%A4llande+sfs&p={}",i);
    let iter = extract_tag(&url, 3, take, LINK_TAG, "href");
    iter
}

/// extract_tag will return an array will all html tags from a url that match the 
/// tag given.
/// ignore and take define the section of the page to select attr defines the 
/// attribute we want to extract from each tag.
/// @param url -- site to visit 
/// @param ignore -- the amount of tags from the first index to ignore, similar to n..
/// @param take -- the amount of tags to select, usually the page size in a paginated list
/// @param tag -- the html tag to select e.g. a.price-link, div.body-text
/// @param attr -- html attribute to select e.g. href
/// @return Vec<String> -- a vector of tags of length take that match the tag and attribute
fn extract_tag(url: &str, ignore: usize, take: usize, tag: &str, attr: &str) -> Vec<String>{
    // Get HTML 
    let mut _ignore = 0;
    let body_future = reqwest::blocking::get(url);
    let body = &body_future.unwrap().text().unwrap();

    // Parse
    let document = Html::parse_document(body);
    let selector = Selector::parse(tag).unwrap();

    let mut iter = document.select(&selector);

    while _ignore < ignore {
        iter.next();
        _ignore += 1;
    }

    let mut res: Vec<String>;

    if attr.len() > 0 {
        res = iter.take(take).map(|x| x.value().attr(attr).unwrap().to_string()).collect();
    }
    else {
        res = iter.map(|x| x.text().next().unwrap().to_string()).collect();
    }

    res
}

/// write_paragraphs will be given a url with a law and return a vector of
/// all paragraphs on the site
/// @param url: &str -- the url to visit
/// @returns Vec<String> -- paragraphs
fn write_paragraphs(url: &str) -> Vec<String> {

    let re: regex::Regex = Regex::new(r"^((\d|\s)*§)").unwrap();
    let mut paragraphs: Vec<String> = vec![];

    println!("{:?}", url);
    let raw_text_1 = extract_tag(url, 0, 1, RAW_TEXT_TAG, "")[0].replace("\t", "");
    let raw_text_1 = raw_text_1.split("\n");

    let mut current:Vec<&str> = vec![]; 
    for line in raw_text_1 {
        let m = re.find(line);
        if line.len() < 2 {
            continue;
        }
        if(!m.is_some()) {
            current.push(line);
        }
        else{
            paragraphs.push(current.join(" "));
            current.clear();
            current.push(line);
        }
    }
    paragraphs.push(current.join(" "));

    paragraphs

}

/// write_full_text will extract the raw text content from a url using the 
/// RAW_TEXT_TAG
/// @params url: &str -- the url to get content from
/// @returns String -- string with text content from the website, no newline or tabs
fn write_full_text(url: &str) -> String{
    let raw_text = extract_tag(url, 0, 1, RAW_TEXT_TAG, "")[0].replace("\t", "");
    let raw_text = raw_text.replace("\n", "").replace("\t", "");

    raw_text
}

fn main() {

    // Regex will match with strings that start with digit followed by '§'
    let re = Regex::new(r"^((\d|\s)*§)").unwrap();

    let links = main_page_links(1, PAGE_SIZE);

    for link in links {
        let url = &extract_tag(&link, 3, 1, LINK_TAG, "href")[0];
        //let text = write_full_text(url);
        let ps = write_paragraphs(url);

        println!("{:?}", ps);
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_link_is_correct() {
        let p1 = main_page_links(1, PAGE_SIZE);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/tillkannagivande-2023954-av-uppgift-om_sfs-2023-954/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2023714-om-forvarv-och-forvaltning-av_sfs-2023-714/");
    }
    #[test]
    fn is_wrong() {
        let p1 = main_page_links(1, PAGE_SIZE);

        assert_ne!(p1[0], "not a link");
    }
    #[test]
    fn first_link_last_page_is_correct() {
        let p1 = main_page_links(268, PAGE_SIZE);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/for-riksbankens-styrelse-och-forvaltning_c5s0riksb/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/byggningabalk-17360123-1_sfs-1736-0123%201/");
    }

    #[test]
    fn link_random_page_is_correct() {
        let p1 = main_page_links(123, PAGE_SIZE);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/forordning-2006421-om-sakerhet-i-vagtunnlar_sfs-2006-421/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2006228-med-sarskilda-bestammelser-om_sfs-2006-228/");
        assert_eq!(p1[10], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2006323-om-utbyte-av-sprutor-och-kanyler_sfs-2006-323/");
    }

    #[test]
    fn raw_text_is_correct_random_law() {
        let url = "http://rkrattsbaser.gov.se/sfst?bet=2023:903"; 
        let text = write_full_text(url);
        let correct = fs::read_to_string("/home/filip/Dokument/lawgpt/thescraper/src/test/raw_text_test_1.txt")
            .expect("Can't read ./test/raw_text_test_1.txt")
            .replace("\t", "")
            .replace("\n", "");

        assert_le!(text.len(), correct.len());
        assert!(text.contains("Träder i kraft I:2024-04-02"));
        assert!(text.contains("i mån av tillgång på medel, ges till en kommun som efter den 31 december 2023 antar e"));
        assert!(text.contains("Överklagande 15 § Beslut enligt denna förordning får inte överklagas."));
    }

    #[test]
    fn paragraphs_is_correct_random_law() {
        let url = "http://rkrattsbaser.gov.se/sfst?bet=2023:903"; 
        let ps = write_paragraphs(url);
        assert!(ps[1].contains("1 § I denna förordning finns bestämmelser om stöd"));
        assert!(ps[1].contains("- 8 kap. 7 § regeringsformen i fråga om övriga bestämmelser."));
        assert!(ps[15].contains("Beslut enligt denna förordning får inte överklagas."));
    }
}
