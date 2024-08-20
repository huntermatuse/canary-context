use clap::{Arg, Command};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TagContext {
    tag_name: String,
    tag_context: TagDetails,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TagDetails {
    historian_item_id: Option<String>,
    source_item_id: Option<String>,
    oldest_time_stamp: String,
    latest_time_stamp: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Vec<TagContext>,
}

async fn get_tags(client: &Client, canary: &str, api_version: &str, api_token: &str, application: &str, timezone: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let url = format!("{}/{}", canary, api_version);
    let payload = serde_json::json!({
        "application": application,
        "timezone": timezone,
        "apiToken": api_token,
        "path": "",
        "deep": true,
        "search": ""
    });

    let response = client.post(format!("{}/browseTags", url))
        .json(&payload)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let tags = response["tags"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|tag| tag.as_str().map(String::from))
        .collect();

    Ok(tags)
}

async fn get_tag_context(client: &Client, canary: &str, api_version: &str, api_token: &str, tags: Vec<String>) -> Result<Vec<TagContext>, Box<dyn Error>> {
    let url = format!("{}/{}", canary, api_version);
    let payload = serde_json::json!({
        "apiToken": api_token,
        "tags": tags
    });

    let response = client.post(format!("{}/getTagContext", url))
        .json(&payload)
        .send()
        .await?
        .json::<ApiResponse>()
        .await?;

    Ok(response.data)
}

fn save_to_csv(data: &Vec<TagContext>, filename: &str) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(filename)?;
    wtr.write_record(["tag_name", "historian_item_id", "source_item_id", "oldest_time_stamp", "latest_time_stamp"])?;

    for item in data {
        wtr.write_record([
            &item.tag_name,
            item.tag_context.historian_item_id.as_deref().unwrap_or(""),
            item.tag_context.source_item_id.as_deref().unwrap_or(""),
            &item.tag_context.oldest_time_stamp,
            &item.tag_context.latest_time_stamp,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

fn save_to_txt(data: &Vec<TagContext>, filename: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;

    for item in data {
        writeln!(file, "TagName: {}", item.tag_name)?;
        writeln!(file, "  HistorianItemId: {}", item.tag_context.historian_item_id.as_deref().unwrap_or(""))?;
        writeln!(file, "  SourceItemId: {}", item.tag_context.source_item_id.as_deref().unwrap_or(""))?;
        writeln!(file, "  OldestTimeStamp: {}", item.tag_context.oldest_time_stamp)?;
        writeln!(file, "  LatestTimeStamp: {}", item.tag_context.latest_time_stamp)?;
        writeln!(file)?;
    }

    Ok(())
}

fn save_to_json(data: &Vec<TagContext>, filename: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(filename)?;
    serde_json::to_writer_pretty(file, data)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Canary CLI")
        .version("1.0")
        .about("CLI tool to interact with the Canary API")
        .arg(Arg::new("canary")
            .long("canary")
            .value_parser(clap::value_parser!(String))
            .required(true)
            .help("Base URL for the Canary server"))
        .arg(Arg::new("api_version")
            .long("api_version")
            .value_parser(clap::value_parser!(String))
            .default_value("api/v2")
            .help("API version to use"))
        .arg(Arg::new("api_token")
            .long("api_token")
            .value_parser(clap::value_parser!(String))
            .required(true)
            .help("API token for authentication"))
        .arg(Arg::new("application")
            .long("application")
            .value_parser(clap::value_parser!(String))
            .default_value("Postman Test")
            .help("Application name"))
        .arg(Arg::new("timezone")
            .long("timezone")
            .value_parser(clap::value_parser!(String))
            .default_value("Pacific Standard Time")
            .help("Timezone to use"))
        .arg(Arg::new("output_format")
            .long("output_format")
            .value_parser(clap::value_parser!(String))
            .required(true)
            .help("Output format for saving the data"))
        .arg(Arg::new("output_file")
            .long("output_file")
            .value_parser(clap::value_parser!(String))
            .required(true)
            .help("Output file name"))
        .get_matches();

    let canary = matches.get_one::<String>("canary").unwrap();
    let api_version = matches.get_one::<String>("api_version").unwrap();
    let api_token = matches.get_one::<String>("api_token").unwrap();
    let application = matches.get_one::<String>("application").unwrap();
    let timezone = matches.get_one::<String>("timezone").unwrap();
    let output_format = matches.get_one::<String>("output_format").unwrap();
    let output_file = matches.get_one::<String>("output_file").unwrap();

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let tags = get_tags(&client, canary, api_version, api_token, application, timezone).await?;
    if !tags.is_empty() {
        let tag_context_data = get_tag_context(&client, canary, api_version, api_token, tags).await?;

        match output_format.as_str() {
            "csv" => save_to_csv(&tag_context_data, output_file)?,
            "txt" => save_to_txt(&tag_context_data, output_file)?,
            "json" => save_to_json(&tag_context_data, output_file)?,
            _ => unreachable!(),
        }

        println!("Data saved to {} in {} format.", output_file, output_format);
    } else {
        println!("No tags found.");
    }

    Ok(())
}
