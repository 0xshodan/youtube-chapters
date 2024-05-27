use reqwest;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::{collections::BTreeMap, fmt::Debug, io::stdin};

const BASE_URL: &str = "https://yt.lemnoslife.com";

struct ParseError {
    message: String,
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

async fn send_request(id: String) -> Result<Value, reqwest::Error> {
    let resp: Value = reqwest::get(format!("{BASE_URL}/videos?part=chapters&id={id}"))
        .await?
        .json()
        .await?;
    if let Value::Object(err) = &resp["error"] {
        println!("{err:#?}");
    }
    return Ok(resp);
}

fn parse_response(resp: Value) -> Result<BTreeMap<u64, String>, ParseError> {
    if let Some(items) = resp["items"].as_array() {
        if let Some(chapters) = items[0]["chapters"]["chapters"].as_array() {
            return Ok(chapters
                .into_iter()
                .map(|val| {
                    (
                        val["time"].as_u64().unwrap(),
                        val["title"].as_str().unwrap().to_owned(),
                    )
                })
                .collect());
        }
    }
    return Err(ParseError {
        message: "Parsing error!".to_owned(),
    });
}

#[tokio::main]
async fn main() {
    let mut user_input = String::new();
    stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");
    user_input = user_input.trim().into();
    let timestamps = parse_response(send_request(user_input).await.unwrap()).unwrap();
    let mut content = ";FFMETADATA1\n".to_string();
    for (time, title) in timestamps.into_iter() {
        match time {
            0 => content.push_str(&format!(
                "[CHAPTER]\nTIMEBASE=1/10000000\nSTART={time}\ntitle={title}\n"
            )),
            _ => {
                let x = time * 10000000;
                content.push_str(&format!(
                    "END={x}\n[CHAPTER]\nTIMEBASE=1/10000000\nSTART={x}\ntitle={title}\n"
                ))
            }
        }
    }

    let mut file = File::create("chapters.txt").unwrap();
    file.write(content.as_bytes()).unwrap();
}
