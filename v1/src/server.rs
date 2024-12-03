use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use perf_event::{Builder, Group};
use perf_event::events::{Hardware, Software};

const FOLDER: &str = "/mnt/raid0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Start the server
    let addr = "0.0.0.0:5201";
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn({
            async move {

                // Create performance event groups
                let mut total_group = Group::new().unwrap();
                let total_cycles = Builder::new()
                    .group(&mut total_group)
                    .kind(Software::CPU_CLOCK)
                    .build().unwrap();

                let mut create_file_group = Group::new().unwrap();
                let create_file_cycles = Builder::new()
                    .group(&mut create_file_group)
                    .kind(Software::CPU_CLOCK)
                    .build().unwrap();

                let mut write_file_group = Group::new().unwrap();
                let write_file_cycles = Builder::new()
                    .group(&mut write_file_group)
                    .kind(Software::CPU_CLOCK)
                    .build().unwrap();

                let mut sync_file_group = Group::new().unwrap();
                let sync_file_cycles = Builder::new()
                    .group(&mut sync_file_group)
                    .kind(Software::CPU_CLOCK)
                    .build().unwrap();

                let mut network_group = Group::new().unwrap();
                let network_cycles = Builder::new()
                    .group(&mut network_group)
                    .kind(Software::CPU_CLOCK)
                    .build().unwrap();
                let mut buffer = vec![0u8; 64 * 1024 * 1024];

                let mut header = vec![0u8; 256];
                total_group.enable().unwrap(); // Start measuring total CPU cycles
                                               //
                network_group.enable().unwrap(); // Start measuring network I/O CPU cycles
                if socket.read_exact(&mut header).await.is_err() {
                    eprintln!("Failed to read header.");
                    network_group.disable().unwrap();
                    total_group.disable().unwrap();
                    return;
                }
                network_group.disable().unwrap(); // Stop network I/O CPU measurement

                let header_str = String::from_utf8_lossy(&header);
                let header_str = header_str.trim_matches(char::from(0));
                let parts: Vec<&str> = header_str.split('|').collect();
                if parts.len() != 2 {
                    eprintln!("Invalid header format.");
                    total_group.disable().unwrap();
                    return;
                }
                let file_name = parts[0];
                let file_size: u64 = match parts[1].trim().parse() {
                    Ok(size) => size,
                    Err(_) => {
                        eprintln!("Invalid file size in header.");
                        total_group.disable().unwrap();
                        return;
                    }
                };
                println!("Receiving file: {} ({} bytes)", file_name, file_size);

                let file_path = Path::new(FOLDER).join(file_name);
                
                create_file_group.enable().unwrap(); // Start measuring disk I/O CPU cycles
                let mut file = match File::create(&file_path).await {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Failed to create file {:?}: {}", file_path, e);
                        create_file_group.disable().unwrap();
                        total_group.disable().unwrap();
                        return;
                    }
                };
                create_file_group.disable().unwrap(); // Stop disk I/O CPU measurement

                let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
                let writer_task = tokio::spawn(async move {
                    while let Some(data) = rx.recv().await {
                        
                        write_file_group.enable().unwrap(); // Measure disk write cycles
                        if let Err(e) = file.write_all(&data).await {
                            eprintln!("Failed to write to file: {}", e);
                            return;
                        }
                        write_file_group.disable().unwrap();
                    }


                    sync_file_group.enable().unwrap(); // Measure file sync cycles
                    if let Err(e) = file.sync_all().await {
                        eprintln!("Failed to sync file: {}", e);
                    }
                    sync_file_group.disable().unwrap();



                    let write_file_counts = write_file_group.read().unwrap();
                    let sync_file_counts = sync_file_group.read().unwrap();

                    println!("Write File I/O CPU cycles: {}", write_file_counts[&write_file_cycles]);
                    println!("Sync File I/O CPU cycles: {}", sync_file_counts[&sync_file_cycles]);
                });

                let mut received = 0;
                while received < file_size {
                    network_group.enable().unwrap(); // Start network I/O measurement
                    let n = match socket.read(&mut buffer).await {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("Failed to read from socket: {}", e);
                            break;
                        }
                    };
                    network_group.disable().unwrap(); // Stop network I/O measurement

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

                total_group.disable().unwrap(); // Stop measuring total CPU cycles

                let total_counts = total_group.read().unwrap();
                let create_file_counts = create_file_group.read().unwrap();
                let network_counts = network_group.read().unwrap();

                println!(
                    "File {} received successfully and saved to {:?}",
                    file_name, file_path
                );
                println!("Total CPU cycles: {}", total_counts[&total_cycles]);
                println!("Create File I/O CPU cycles: {}", create_file_counts[&create_file_cycles]);
                println!("Network I/O CPU cycles: {}", network_counts[&network_cycles]);
            }
        });
    }
}
