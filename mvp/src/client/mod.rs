use clap::{Parser, ValueEnum};
use rand::Rng;
use reqwest::multipart::{Form, Part};
use reqwest::{Client};
use tokio::fs::{File};
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use std::{sync::Arc};
use std::time::Duration;
use anyhow::{Error, Context, Result};
use tokio::{
    task::{self, JoinHandle},
    time::Instant,
};

use crate::PARQUET_FOLDER;


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    Send,
    Receive,
    Mixed,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: String,

    #[arg(short, long)]
    mode: Mode,

    #[arg(short, long, default_value_t = 60)]
    duration: u64,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let client = Arc::new(Client::new());
    let url = Arc::new(format!("http://{}/parquet", args.ip.to_string()));

    let end_time = Instant::now() + Duration::from_secs(args.duration);
    let mut tasks = Vec::new();

    let mut rng = rand::thread_rng();
    let mut file_name_counter = 0;

    while Instant::now() < end_time {
        file_name_counter += 1;
        let file_name = format!("test{}", file_name_counter);
        let file_path = PathBuf::from(PARQUET_FOLDER).join(&file_name);
        if args.mode == Mode::Send {
            spawn_sender(Arc::clone(&client), Arc::clone(&url), &mut tasks, file_name, file_path);
        } else if args.mode == Mode::Receive {
            spawn_receiver(Arc::clone(&client), Arc::clone(&url), &mut tasks, file_name);
        } else {
            let num: f64 = rng.gen();
            if num < 0.8 {
                spawn_sender(Arc::clone(&client), Arc::clone(&url), &mut tasks, file_name, file_path);
            } else {
                spawn_receiver(Arc::clone(&client), Arc::clone(&url), &mut tasks, file_name);
            }
        }
    }

    for task in tasks {
        if let Err(err) = task.await {
            eprintln!("Error in request: {:?}", err);
        }
    }
    Ok(())
}

fn spawn_sender(
    client: Arc<Client>,
    url: Arc<String>,
    tasks: &mut Vec<JoinHandle<Result<(), Error>>>,
    file_name: String,
    file_path: PathBuf,
) {
    let task = task::spawn(async move { send_data_request(&client, &url, &file_name, &file_path).await });
    tasks.push(task);
}

fn spawn_receiver(
    client: Arc<Client>,
    url: Arc<String>,
    tasks: &mut Vec<JoinHandle<Result<(), Error>>>,
    file_name: String,
) {
    let task = task::spawn(async move { receive_data_request(&client, &url, &file_name).await });
    tasks.push(task);
}

async fn send_data_request(client: &Client, url: &str, file_name: &str, file_path: &PathBuf) -> Result<()> {
    let mut file: File = File::open(file_path)
        .await
        .with_context(|| format!("Failed opening file at: {:?}", file_path))?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)
        .await
        .with_context(|| format!("Failed reading file at: {:?}", file_path))?;

    let part = Part::bytes(file_contents)
        .file_name(file_name.to_string());

    let form = Form::new()
        .part("file", part);
    
    let response = client
        .put(url)
        .multipart(form)
        .send()
        .await?;

    println!("POST Status: {}", response.status());
    Ok(())
}

async fn receive_data_request(client: &Client, url: &str, file_name: &str) -> Result<()> {
    let url = format!("{}/{}", &url, &file_name);

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to send GET request")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Request failed with status {}", response.status()));
    }
    Ok(())
}
