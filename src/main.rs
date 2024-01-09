use std::error::Error;
use futures::executor::block_on;
use scraper::{Html, Selector};
use scraper::html::Select;
use std::iter::Enumerate;
use scraper::ElementRef;

const PAGE_SIZE:usize = 20;
const LINK_TAG: &str = "a.sc-3189427c-0";
const RAW_TEXT_TAG: &str = "div.body-text";

fn main_page_links(i: u64, take: usize) -> Vec<String>{
    // Get HTML 
    let url = format!("https://www.riksdagen.se/sv/sok/?doktyp=sfs&dokstat=g%C3%A4llande+sfs&p={}",i);
    let iter = extract_tag(&url, 1, take, LINK_TAG, "href");
    iter
}

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
        res = iter.take(take).map(|x| x.value().attr("href").unwrap().to_string()).collect();
    }
    else {
        res = iter.map(|x| x.text().next().unwrap().to_string()).collect();
    }

    res
}

fn main() {
    let mut body: &str = "";
    let links               = main_page_links(1, PAGE_SIZE);
    for link in links{
        let url = extract_tag(&link, 1, 1, LINK_TAG, "href");
        let raw_text_1 = extract_tag(&url[0], 0, 1, RAW_TEXT_TAG, "");
        println!("{:?}\n", raw_text_1[0].replace("\n", "").replace("\t", ""));
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_link_is_correct() {
        let p1 = main_page_links(1);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/tillkannagivande-2023954-av-uppgift-om_sfs-2023-954/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2023714-om-forvarv-och-forvaltning-av_sfs-2023-714/");
    }
    #[test]
    fn is_wrong() {
        let p1 = main_page_links(1);

        assert_ne!(p1[0], "not a link");
    }
    #[test]
    fn first_link_last_page_is_correct() {
        let p1 = main_page_links(268);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/for-riksbankens-styrelse-och-forvaltning_c5s0riksb/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/byggningabalk-17360123-1_sfs-1736-0123%201/");
    }

    #[test]
    fn link_random_page_is_correct() {
        let p1 = main_page_links(123);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/forordning-2006421-om-sakerhet-i-vagtunnlar_sfs-2006-421/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2006228-med-sarskilda-bestammelser-om_sfs-2006-228/");
        assert_eq!(p1[10], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2006323-om-utbyte-av-sprutor-och-kanyler_sfs-2006-323/");
    }
}
