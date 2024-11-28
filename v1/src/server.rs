use std::path::Path;
use perf_event::events::Hardware;
use perf_event::{Builder, Group};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

const FOLDER: &str = "/mnt/raid0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Create performance event groups
    let mut total_group = Group::new()?;
    let total_cycles = Builder::new()
        .group(&mut total_group)
        .kind(Hardware::CPU_CYCLES)
        .build()?;

    let mut disk_group = Group::new()?;
    let disk_cycles = Builder::new()
        .group(&mut disk_group)
        .kind(Hardware::CPU_CYCLES)
        .build()?;

    let mut network_group = Group::new()?;
    let network_cycles = Builder::new()
        .group(&mut network_group)
        .kind(Hardware::CPU_CYCLES)
        .build()?;



    let addr = "0.0.0.0:5201";
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;


        tokio::spawn(

            async move {
            let mut buffer = vec![0u8; 64 * 1024];

            let mut header = vec![0u8; 256];
            if socket.read_exact(&mut header).await.is_err() {
                eprintln!("Failed to read header.");
                return;
            }
            let header_str = String::from_utf8_lossy(&header);
            let header_str = header_str.trim_matches(char::from(0));
            let parts: Vec<&str> = header_str.split('|').collect();
            if parts.len() != 2 {
                eprintln!("Invalid header format.");
                return;
            }
            let file_name = parts[0];
            let file_size: u64 = match parts[1].trim().parse() {
                Ok(size) => size,
                Err(_) => {
                    eprintln!("Invalid file size in header.");
                    return;
                }
            };
            println!("Receiving file: {} ({} bytes)", file_name, file_size);

            let file_path = Path::new(FOLDER).join(file_name);
            let mut file = match File::create(&file_path).await {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to create file {:?}: {}", file_path, e);
                    return;
                }
            };

            let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
            let writer_task = tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    if let Err(e) = file.write_all(&data).await {
                        eprintln!("Failed to write to file: {}", e);
                        return;
                    }
                }

                if let Err(e) = file.sync_all().await {
                    eprintln!("Failed to sync file: {}", e);
                }
            });

            let mut received = 0;
            while received < file_size {
                let n = match socket.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        break;
                    }
                };

                if tx.send(buffer[..n].to_vec()).await.is_err() {
                    eprintln!("Failed to send data to writer task.");
                    return;
                }

                received += n as u64;
            }

            drop(tx);

            if let Err(e) = writer_task.await {
                eprintln!("Writer task failed: {:?}", e);
            }

            println!(
                "File {} received successfully and saved to {:?}",
                file_name, file_path
            );
        });
    }
}
