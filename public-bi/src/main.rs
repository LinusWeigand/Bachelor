use async_compression::tokio::bufread::BzDecoder;
use duckdb::Connection;
use futures_util::StreamExt;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::fs::read_dir;
use std::path::PathBuf;
use std::{error::Error, fs};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

const URL: &str = "https://event.cwi.nl/da/PublicBIbenchmark";
const REPO: &str = "/Users/linusweigand/Downloads/public_bi_benchmark/benchmark/";
const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let openai_api_key = std::env::var("OPENAI_API_KEY")
        .expect("ENV VAR OPENAI_API_KEY must be set!");

    let conn = Connection::open("database.duckdb")?;
    let mut database_names: Vec<String> = Vec::new();
    let current_dir = PathBuf::from(REPO);


    let index = tokio::fs::read_to_string("index.txt").await?;
    let index = index.trim().parse::<u32>().unwrap();
    let mut counter = 0;
    for entry in fs::read_dir(&current_dir)? {
        counter += 1;
        if counter != index {
            continue;
        }
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Save name
        let name = path.file_name().unwrap().to_string_lossy();
        database_names.push(name.to_string());

        // Download files
        let url = format!("{}/{}", URL, name);
        println!("url: {}", url);
        for i in 1..100 {
            let filename = format!("{}_{}.csv", name, i);
            let compressed_file_name = format!("{}.bz2", filename);


            let request_url = format!("{}/{}", url, compressed_file_name);

            let response = reqwest::get(&request_url).await?;

            if response.status() != StatusCode::OK {
                println!("All Files downloaded.");
                break;
            }

            // Stream to File
            println!("Streaming to file...");
            let compressed_filepath = path.clone().join(compressed_file_name);
            let mut compressed_file = File::create(&compressed_filepath).await?;

            let mut content = response.bytes_stream();
            while let Some(chunk) = content.next().await {
                let chunk = chunk?;
                compressed_file.write_all(&chunk).await?;
            }
            println!("Streamed file to disk");


            // Decompress
            let filepath = path.clone().join(filename);

            let compressed_file = File::open(&compressed_filepath).await?;
            let mut file = File::create(filepath).await?;

            let reader = BufReader::new(compressed_file);
            let mut decoder = BzDecoder::new(reader);
            let mut buffer = [0u8; 8192];

            println!("Decoding file...");
            loop {
                let bytes_read = decoder.read(&mut buffer).await?;
                if bytes_read == 0 {
                    break;
                }

                file.write_all(&buffer[..bytes_read]).await?;
            }
            println!("Decoding done");
        }

        // Create Tables & insert data
        let mut schemas: Vec<String> = Vec::new();
        let mut table_names: Vec<String> = Vec::new();
        let tables_dir = path.clone().join("tables");

        for table in read_dir(tables_dir)? {
            let table = table?.path();
            let schema_content = tokio::fs::read_to_string(&table).await?;
            conn.execute_batch(&schema_content)?;
            schemas.push(schema_content);

            let table_name = table
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.split('.').next().unwrap())
                .unwrap();
            table_names.push(table_name.to_string());
            let csv_file_path = path.clone().join(format!("{}.csv", table_name));
            let copy_command = format!(
                "COPY \"{}\" FROM '{}' (DELIMITER '|', HEADER FALSE, NULL 'null');",
                table_name,
                csv_file_path.display()
            );
            println!("Copying data into table...");
            conn.execute(&copy_command, [])?;
            println!("Copying finished");
        }

        // Execute Queries
        println!("Executing queries");
        let query_dir = path.clone().join("queries");

        for query in read_dir(query_dir)? {
            let query = query?.path();
            println!("query: {:?}", query);

            let mut query_content = tokio::fs::read_to_string(&query).await?;
            query_content = format!("{}\n{}", "EXPLAIN ANALYSE", query_content);

            let mut stmt = conn.prepare(&query_content)?;

            let mut rows = stmt.query([])?;
            let mut explanation = String::new();

            while let Some(row) = rows.next()? {
                let mut row_data = Vec::new();
                for i in 0.. {
                    match row.get::<_, String>(i) {
                        Ok(value) => row_data.push(value),
                        Err(_) => break,
                    }
                }
                explanation.push_str(&row_data.join("\t"));
                explanation.push('\n');
            }
            let mut query_filename = query
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap()
                .to_string();
            query_filename = format!("{}.txt", query_filename);

            let mut query_file = File::create(&query_filename).await?;
            query_file.write_all(explanation.as_bytes()).await?;

            let system_prompt = tokio::fs::read_to_string("system_prompt.txt").await?;

            let mut user_prompt = explanation;
            for schema in &schemas {
                user_prompt.push_str("\n\n");
                user_prompt.push_str(&schema);
            }

            let messages = vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ];
            let model = "gpt-4o".to_string();
            // let model = "gpt-3.5-turbo".to_string();
            let request_body = ChatRequest {
                model,
                messages,
                max_tokens: 100,
                temperature: 0.7,
            };


            let client = Client::new();
            let response = client
                .post(OPENAI_URL)
                .header("Authorization", format!("Bearer {}", openai_api_key))
                .json(&request_body)
                .send()
                .await?;


            if !response.status().is_success() {
                println!("Failed to get a response from openai: {}", response.text().await?);
                break;
            }

            let api_response: ChatResponse = response.json().await?;
            let content = &api_response.choices[0].message.content;

            let result: StrucuturedResponse = serde_json::from_str(content)?;
            println!("Extracted Response: {:?}", result);

            let row_selectivity = result.used_rows as f64 / result.total_rows as f64;
            let col_selectivity = result.used_cols as f64 / result.total_cols as f64;
            let total_selectivity = row_selectivity * col_selectivity;

            //Append to result
            let mut result_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("result.csv")
                .await?;

            result_file.write_all(
                format!("{},{},{},{},{},{},{},{}\n", 
                    result.table_name, 
                    result.used_rows, 
                    result.total_rows, 
                    row_selectivity, 
                    result.used_cols, 
                    result.total_cols, 
                    col_selectivity,
                    total_selectivity
                    )
                .as_bytes()).await?;
            result_file.flush().await?;
        }
        // Delete csv and bz2 files
        let mut entries = tokio::fs::read_dir(&path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(extension) = path.extension() {
                if extension != "csv" && extension != "bz2" {
                    continue;
                }

                println!("Deleting file: {:?}", path);
                tokio::fs::remove_file(path).await?;
            }

        }

        // Drop tables
        for table_name in table_names {
            println!("Droping Table: {}", table_name);
            let drop_table_query = format!("DROP TABLE IF EXISTS {}", table_name);
            conn.execute(&drop_table_query, [])?;
        }

        // Update index;
        let mut index_file = OpenOptions::new()
            .write(true)
            .open("index.txt")
            .await?;

        index_file.write_u32(index).await?;
        index_file.flush().await?;
    }

    Ok(())
}



#[derive(Deserialize, Debug)]
struct StrucuturedResponse {
    table_name: String,
    total_rows: u64,
    used_rows: u64,
    total_cols: u64,
    used_cols: u64,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: usize,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize, Debug)]
struct MessageContent {
    content: String,
}
