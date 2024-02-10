use std::fs::File;
use std::io::prelude::*;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::embedding::EmbeddingRequest;
use openai_api_rs::v1::embedding::EmbeddingResponse;
use std::fs;
use std::env;

mod paragrapher;

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
    //let start = Instant::now();

    // let start_batch = 8;

    paragrapher::demo();

    //let handle_1 = thread::spawn(move || {
    //    for n in start_batch..26u64 {
    //        let _ = scrape_page(2 + 10 * n);
    //    }
    //});
    //let handle_2 = thread::spawn(move || {
    //    for n in start_batch..26u64 {
    //        let _ = scrape_page(3 + 10 * n);
    //    }
    //});
    //let handle_3 = thread::spawn(move || {
    //    for n in start_batch..26u64 {
    //        let _ = scrape_page(4 + 10*n);
    //    }
    //});
    //let handle_4 = thread::spawn(move || {
    //    for n in start_batch..26u64 {
    //        let _ = scrape_page(5 + 10*n);
    //    }
    //});
    //let handle_5 = thread::spawn(move || {
    //    for n in start_batch..26u64 {
    //        let _ = scrape_page(6 + 10*n);
    //    }
    //});
    //let handle_6 = thread::spawn(move || {
    //    for n in start_batch..25u64 {
    //        let _ = scrape_page(7 + 10*n);
    //    }
    //});
    //let handle_7 = thread::spawn(move || {
    //    for n in start_batch..25u64 {
    //        let _ = scrape_page(8 + 10*n);
    //    }
    //});
    //let handle_8 = thread::spawn(move || {
    //    for n in start_batch..25u64 {
    //        let _ = scrape_page(9 + 10*n);
    //    }
    //});
    //let handle_9 = thread::spawn(move || {
    //    for n in start_batch..25u64 {
    //        let _ = scrape_page(10 + 10*n);
    //    }
    //});

    //for n in start_batch..26u64 {
    //    println!("---------------------------------------------------------------");
    //    println!("Getting batch {}", n);
    //    println!("---------------------------------------------------------------");

    //    let _ = scrape_page(1+ 10*n);

    //}

    //handle_1.join().expect("Could not unwrap thread");
    //handle_2.join().expect("Could not unwrap thread");
    //handle_3.join().expect("Could not unwrap thread");
    //handle_4.join().expect("Could not unwrap thread");
    //handle_5.join().expect("Could not unwrap thread");
    ////handle_6.join().expect("Could not unwrap thread");
    //handle_7.join().expect("Could not unwrap thread");
    //handle_8.join().expect("Could not unwrap thread");
    //handle_9.join().expect("Could not unwrap thread");

    //let duration = start.elapsed();


    //println!("Time elapsed {:?}", duration);
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
