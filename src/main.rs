use std::error::Error;
use futures::executor::block_on;
use scraper::{Html, Selector};
use scraper::html::Select;
use std::iter::Enumerate;

fn parse_page(i: u64) -> Vec<String>{
    // Get HTML 
    let url = format!("https://www.riksdagen.se/sv/sok/?doktyp=sfs&dokstat=g%C3%A4llande+sfs&p={}",i);
    let body_future = reqwest::blocking::get(url);
    let body = &body_future.unwrap().text().unwrap();

    // Parse
    let document = Html::parse_document(body);
    let selector = Selector::parse("a.sc-3189427c-0").unwrap();

    let mut iter = document.select(&selector);

    iter.next();

    let mut res: Vec<String> = iter.take(20).map(|x| x.value().attr("href").unwrap().to_string()).collect();

    res
}

fn main() {
    let mut body: &str = "";
    let links = parse_page(1);
    println!("{:?}", links);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_link_is_correct() {
        let p1 = parse_page(1);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/tillkannagivande-2023954-av-uppgift-om_sfs-2023-954/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2023714-om-forvarv-och-forvaltning-av_sfs-2023-714/");
    }
    #[test]
    fn is_wrong() {
        let p1 = parse_page(1);

        assert_ne!(p1[0], "not a link");
    }
    #[test]
    fn first_link_last_page_is_correct() {
        let p1 = parse_page(268);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/for-riksbankens-styrelse-och-forvaltning_c5s0riksb/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/byggningabalk-17360123-1_sfs-1736-0123%201/");
    }

    #[test]
    fn link_random_page_is_correct() {
        let p1 = parse_page(123);

        assert_eq!(p1[0], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/forordning-2006421-om-sakerhet-i-vagtunnlar_sfs-2006-421/");
        assert_eq!(p1[19], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2006228-med-sarskilda-bestammelser-om_sfs-2006-228/");
        assert_eq!(p1[10], "https://www.riksdagen.se/sv/dokument-och-lagar/dokument/svensk-forfattningssamling/lag-2006323-om-utbyte-av-sprutor-och-kanyler_sfs-2006-323/");
    }
}
