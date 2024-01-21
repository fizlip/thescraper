use std::thread;
use scraper::{Html, Selector};
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::time::{Instant};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::embedding::EmbeddingRequest;
use openai_api_rs::v1::embedding::EmbeddingResponse;
use std::fs;
use std::env;


// Constants                                                               
const PAGE_SIZE:usize = 20;
const LINK_TAG: &str = "a.sc-3189427c-0";
const RAW_TEXT_TAG: &str = "div.body-text";
const IGNORE: usize = 1;

/// main_page_links will retrieve all law links from www.riksdagen.se
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

    let re: regex::Regex = Regex::new(r"^((\d|\s)*§)").unwrap();
    let mut paragraphs: Vec<String> = vec![];

    let raw_text_1 = extract_tag(url, 0, 1, RAW_TEXT_TAG, "");
    if raw_text_1.len() == 0 {
        println!("Error for {:?}", url);
        return vec![];
    }
    let raw_text_1 = raw_text_1[0].replace("\t", "");
    let raw_text_1 = raw_text_1.split("\n");

    let mut current:Vec<&str> = vec![]; 
    for line in raw_text_1 {
        let m = re.find(line);
        if line.len() < 2 {
            continue;
        }
        if !m.is_some() {
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

/// scrape_page will create .txt files for all paragraphs and laws on the 
/// given page
fn scrape_page(page: u64) -> std::io::Result<()>{
    let law_id_regex = Regex::new(r"\d\d\d\d:\d*").unwrap();

    let links = main_page_links(page, PAGE_SIZE);

    for link in links {
        let url = &extract_tag(&link, IGNORE, 1, LINK_TAG, "href");
        if url.len() == 0 {
            continue
        }
        
        let url = &url[0];

        let law_id = law_id_regex.find(url).expect(&format!("error on url: {}", url)).as_str().replace(":", "_");

        // Create file
        let f_name = format!("/home/filip/Dokument/lawgpt/laws/raw/{}.txt", law_id);
        println!("Writing {}", f_name);

        let mut file_raw = File::create(f_name)?;

        // Get data
        let ps = write_paragraphs(url);
        let raw = write_full_text(url);

        if raw.len() > 0 {
            file_raw.write_all(raw.as_bytes())?;
        }

        for (i,p) in ps.into_iter().enumerate() {
            let f_name_p = format!("/home/filip/Dokument/lawgpt/laws/paragraphs/{}-{}.txt", law_id, i);
            let mut file_paragraph = File::create(f_name_p)?;
            file_paragraph.write_all(p.as_bytes())?;
            create_embedding("/home/filip/Dokument/lawgpt/laws/embeddings/{}-{}.txt", 0);
        }
    }

    Ok(())
}


fn request_embedding(text: String) -> Option<EmbeddingResponse> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());

    // Openai does not allow strings that are too long
    if text.len() > 8191 {
        println!("Input too long");
        return None;
    } 
    let req = EmbeddingRequest::new(
        "text-embedding-ada-002".to_string(),
        text.to_string(),
    );
    let response =
        client.embedding(req).ok()?;
        Some(response)
}

fn create_embedding(fname: &str, tries: u64) -> Result<(), Box<dyn std::error::Error>> {
    let paragraph = fs::read_to_string(fname).expect("Could not read file");

    let mut embedding:String = match request_embedding(paragraph) {
        Some(v) => v.data[0]
            .embedding
            .iter()
            .map(|v| v.to_string() + ",")
            .collect(),
        None => String::from(""),
    };

    if embedding.len() < 10 {
        if tries > 3 {
            println!("Failed to embed: {}", fname);
            ()
        }
        else {
            println!("Could not get trying again");
            return create_embedding(fname, tries + 1);
        }
    }

    embedding.pop();

    let new_fname = fname.replace("laws", "embeddings");
    let mut embedded_paragraph = File::create(new_fname.clone())?;
    println!("writing in {}", new_fname.clone());
    embedded_paragraph.write_all(embedding.as_bytes())?;

    Ok(())
}


fn main() -> std::io::Result<()>{
    let start = Instant::now();

    let start_batch = 8;

    let handle_1 = thread::spawn(move || {
        for n in start_batch..26u64 {
            let _ = scrape_page(2 + 10 * n);
        }
    });
    let handle_2 = thread::spawn(move || {
        for n in start_batch..26u64 {
            let _ = scrape_page(3 + 10 * n);
        }
    });
    let handle_3 = thread::spawn(move || {
        for n in start_batch..26u64 {
            let _ = scrape_page(4 + 10*n);
        }
    });
    let handle_4 = thread::spawn(move || {
        for n in start_batch..26u64 {
            let _ = scrape_page(5 + 10*n);
        }
    });
    let handle_5 = thread::spawn(move || {
        for n in start_batch..26u64 {
            let _ = scrape_page(6 + 10*n);
        }
    });
    let handle_6 = thread::spawn(move || {
        for n in start_batch..25u64 {
            let _ = scrape_page(7 + 10*n);
        }
    });
    let handle_7 = thread::spawn(move || {
        for n in start_batch..25u64 {
            let _ = scrape_page(8 + 10*n);
        }
    });
    let handle_8 = thread::spawn(move || {
        for n in start_batch..25u64 {
            let _ = scrape_page(9 + 10*n);
        }
    });
    let handle_9 = thread::spawn(move || {
        for n in start_batch..25u64 {
            let _ = scrape_page(10 + 10*n);
        }
    });

    for n in start_batch..26u64 {
        println!("---------------------------------------------------------------");
        println!("Getting batch {}", n);
        println!("---------------------------------------------------------------");

        let _ = scrape_page(1+ 10*n);

    }

    handle_1.join().expect("Could not unwrap thread");
    handle_2.join().expect("Could not unwrap thread");
    handle_3.join().expect("Could not unwrap thread");
    handle_4.join().expect("Could not unwrap thread");
    handle_5.join().expect("Could not unwrap thread");
    //handle_6.join().expect("Could not unwrap thread");
    handle_7.join().expect("Could not unwrap thread");
    handle_8.join().expect("Could not unwrap thread");
    handle_9.join().expect("Could not unwrap thread");

    let duration = start.elapsed();


    println!("Time elapsed {:?}", duration);
    Ok(())
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
