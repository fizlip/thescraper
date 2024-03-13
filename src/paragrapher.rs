use scraper::{Html, Selector};
use regex::Regex;
use std::collections::HashMap;

// Constants (current hardcoded) 
const PAGE_SIZE:usize = 20;
const LINK_TAG: &str = "a.sc-3189427c-0";
const RAW_TEXT_TAG: &str = "div.body-text";
const IGNORE: usize = 1;

/// main_page_links (S1) will retrieve all law links from www.riksdagen.se
/// @params i -- the page to retrieve
/// @params take -- the page size
/// @returns Vec<String> -- vector of links to all laws on the page
fn main_page_links(i: u64, take: usize) -> Vec<String>{
    // Get HTML 
    let url = format!("https://www.riksdagen.se/sv/sok/?doktyp=sfs&dokstat=g%C3%A4llande+sfs&p={}",i);
    let iter = extract_tag(&url, IGNORE, take, LINK_TAG, "href");
    iter
}

/// extract_tag will return an array with all html tags from a url that match the 
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
    let body = match body_future {
        Ok(b) => b.text().unwrap(),
        Err(_) => String::from(""),
    };

    if body.len() == 0 {
        return vec![];
    }

    let body = &body;

    // Parse
    let document = Html::parse_document(body);
    let selector = Selector::parse(tag).unwrap();

    let mut iter = document.select(&selector);

    while _ignore < ignore {
        iter.next();
        _ignore += 1;
    }

    let res: Vec<String>;

    if attr.len() > 0 {
        res = iter.take(take).map(|x| x.value().attr(attr).unwrap().to_string()).collect();
    }
    else {
        match iter.take(1).collect::<Vec<_>>().first() {
            Some(v) => res = match v.text().next() {
                Some(val) => vec![val.to_string()],
                None => vec![]
            },
            None => res = vec![],
        }
    }

    res
}

/// write_paragraphs will be given a url with a law and return a vector of
/// all paragraphs on the site
/// @param url: &str -- the url to visit
/// @returns Vec<String> -- paragraphs
fn write_paragraphs(url: &str) -> Vec<String> {

    let re: regex::Regex = Regex::new(r"(?m)^\n(\d|\s|)*§").unwrap();
    //let re: regex::Regex = Regex::new(r" ").unwrap();
    let mut paragraphs: Vec<String> = vec![];

    let raw_text_1 = extract_tag(url, 0, 1, RAW_TEXT_TAG, "");
    if raw_text_1.len() == 0 {
        println!("Error for {:?}", url);
        return vec![];
    }
    let raw_text_1 = raw_text_1[0].replace("\t", "");
    let pre_processed_paragraphs: Vec<String> = re
        .split(&raw_text_1)
        .collect::<Vec<_>>()
        .into_iter()
        .map(|x| String::from(x)).collect();

    for (i, p) in pre_processed_paragraphs.into_iter().enumerate() {

        let mut s:String = format!("{} §", i).to_owned();
        s.push_str(&p.replace("\n", ""));

        paragraphs.push(s);
    }

    paragraphs

}

/// write_full_text will extract the raw text content from a url using the 
/// RAW_TEXT_TAG this function is used to write the full law into a text file
/// @params url: &str -- the url to get content from
/// @returns String -- string with text content from the website, no newline or tabs
fn write_full_text(url: &str) -> String{
    let raw_text = extract_tag(url, 0, 1, RAW_TEXT_TAG, "");
    if raw_text.len() == 0 {
        return String::from("");
    }
    let raw_text = raw_text[0].replace("\t", "");
    let raw_text = raw_text.replace("\n", "").replace("\t", "");

    raw_text
}

/// scrape_page will a 2d vector containing all paragraphs for each url on the 
/// given page
pub fn scrape_page(page: u64) -> std::io::Result<HashMap<String, Vec<String>>>{
    let law_id_regex = Regex::new(r"\d\d\d\d:\d*").unwrap();

    let links = main_page_links(page, PAGE_SIZE);

    let mut res = HashMap::new();

    if links.len() == 0 {
        return Ok(res);
    }

    for link in links {
        let url = &extract_tag(&link, IGNORE, 1, LINK_TAG, "href");
        if url.len() == 0 {
            continue
        }
        
        let url = &url[0];

        if law_id_regex.is_match(url) {
            let law_id = law_id_regex.find(url).expect("URL error").as_str().replace(":", "_");
            let ps = write_paragraphs(url);
            res.insert(String::from(law_id), ps);
        }
        else{
            continue
        }
    }

    Ok(res)
}

