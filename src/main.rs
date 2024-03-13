use std::thread;
use std::time::Duration;

mod paragrapher;
mod embedder;
mod uploader;

fn create_embedding_files(p:u64) -> std::io::Result<()> {
    let res = paragrapher::scrape_page(p)?;

    if res.len() == 0 {
        thread::sleep(Duration::from_secs(1000));
    }

    for (law_id, ps) in &res {
        for (i,p) in ps.into_iter().enumerate() {
            //let emb = embedder::create_embedding(String::from(p), 0).expect("Could not create embedding");
            uploader::write_file_local(&format!("/home/filip/Dokument/lawgpt/laws/paragraphs/{}-{}.txt", law_id, i), p.to_string());
        }
    }

    Ok(())
} 

fn main() -> std::io::Result<()>{
    //let start = Instant::now();

    // let start_batch = 8;
    let mut ts = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            for j in 25..26 {
                println!("Doing page {}", i + 10*j);
                let _ = create_embedding_files(i + 10*j);
            }
        });
        ts.push(handle)
    }

    for t in ts {
        t.join().expect("Could not unwrap thread");
    }

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
