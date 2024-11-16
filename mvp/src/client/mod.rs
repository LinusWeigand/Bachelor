use anyhow::{Context, Error, Result};
use clap::{Parser, ValueEnum};
use rand::Rng;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::{
    task::{self, JoinHandle},
    time::Instant,
};

const PARQUET_FOLDER: &str = "./parquet_files/";
const MIX_RATIO: f64 = 0.8;

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

    #[arg(short, long, default_value_t = 1)]
    duration: u64,

    #[arg(short, long, default_value_t = 1)]
    parallel_clients: u128,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let client = Arc::new(Client::new());
    let url = Arc::new(format!("http://{}/parquet", args.ip.to_string()));

    let file_path = PathBuf::from(PARQUET_FOLDER).join("test_file.parquet");
    let mut file: File = File::open(&file_path)
        .await
        .with_context(|| format!("Failed opening file at: {:?}", file_path))?;
    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents)
        .await
        .with_context(|| format!("Failed reading file at: {:?}", file_path))?;

    let mut client_tasks = Vec::new();

    let offset: u128 = args.parallel_clients * 1000 * 1000 * args.duration as u128;
    let mut cur_offset: u128 = 0;

    for _ in 0..args.parallel_clients {
        let client_clone = Arc::clone(&client);
        let url_clone = Arc::clone(&url);
        let mode = args.mode;
        let duration = Duration::from_secs(args.duration);
        let file_contents = file_contents.clone();

        let client_task = task::spawn(async move {
            spawn_client(
                client_clone,
                url_clone,
                cur_offset,
                mode,
                duration,
                file_contents,
            )
            .await
        });
        client_tasks.push(client_task);
        cur_offset += offset;
    }
    for client_task in client_tasks {
        if let Err(err) = client_task.await {
            eprintln!("Error in request: {:?}", err);
        }
    }

    Ok(())
}

async fn spawn_client(
    client: Arc<Client>,
    url: Arc<String>,
    file_counter_start: u128,
    mode: Mode,
    duration: Duration,
    file_contents: Vec<u8>,
) {
    let mut file_name_counter = file_counter_start;

    let end_time = Instant::now() + duration;
    while Instant::now() < end_time {
        let file_name = format!("test_file{}.parquet", file_name_counter);
        let task = 
            match mode {
            Mode::Send => spawn_sender(
                Arc::clone(&client),
                Arc::clone(&url),
                file_name,
                file_contents.clone(),
            ),
            Mode::Receive => {
                spawn_receiver(Arc::clone(&client), Arc::clone(&url), file_name)
            }
            Mode::Mixed => {
                return if sample_bernouli_var(MIX_RATIO) {
                    spawn_sender(
                        Arc::clone(&client),
                        Arc::clone(&url),
                        file_name,
                        file_contents.clone(),
                    );
                } else {
                    spawn_receiver(Arc::clone(&client), Arc::clone(&url), file_name);
                }
            }
        };
        if let Err(err) = task.await {
            eprintln!("Error in request: {:?}", err);
        }
        file_name_counter += 1;
    }
}

fn sample_bernouli_var(theta: f64) -> bool {
    let mut rng = rand::thread_rng();
    let num: f64 = rng.gen();
    num > theta
}

fn spawn_sender (
    client: Arc<Client>,
    url: Arc<String>,
    file_name: String,
    file_contents: Vec<u8>,
) -> JoinHandle<Result<(), anyhow::Error>> {
    task::spawn(
        async move { send_data_request(&client, &url, &file_name, file_contents).await },
    )
}

fn spawn_receiver(
    client: Arc<Client>,
    url: Arc<String>,
    file_name: String,
) -> JoinHandle<Result<(), anyhow::Error>> {
    task::spawn(async move { receive_data_request(&client, &url, &file_name).await })
}

async fn send_data_request(
    client: &Client,
    url: &str,
    file_name: &str,
    file_contents: Vec<u8>,
) -> Result<()> {
    let url = format!("{}/{}", &url, &file_name);

    let part = Part::bytes(file_contents).file_name(file_name.to_string());
    let form = Form::new().part("file", part);
    let _response = client.put(url).multipart(form).send().await?;

    println!("Got Response to PUT request");
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
        return Err(anyhow::anyhow!(
            "Request failed with status {}",
            response.status()
        ));
    }
    println!("Got Response to GET request");
    Ok(())
}
