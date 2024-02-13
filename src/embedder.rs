use std::fs::File;
use std::io::prelude::*;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::embedding::EmbeddingRequest;
use openai_api_rs::v1::embedding::EmbeddingResponse;
use std::fs;
use std::env;

pub fn request_oai_embedding(text: String) -> Option<EmbeddingResponse> {
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

pub fn create_embedding(paragraph: String, tries: u64) -> Result<String, Box<dyn std::error::Error>> {

    let mut embedding:String = match request_oai_embedding(paragraph.clone()) {
        Some(v) => v.data[0]
            .embedding
            .iter()
            .map(|v| v.to_string() + ",")
            .collect(),
        None => String::from(""),
    };

    if embedding.len() < 10 {
        if tries > 3 {
            println!("Failed to embed");
            ()
        }
        else {
            println!("Could not get trying again");
            return create_embedding(paragraph.clone(), tries + 1);
        }
    }

    embedding.pop();

    //let new_fname = fname.replace("laws", "embeddings");
    //let mut embedded_paragraph = File::create(new_fname.clone())?;
    //println!("writing in {}", new_fname.clone());
    //embedded_paragraph.write_all(embedding.as_bytes())?;

    Ok(embedding)
}
