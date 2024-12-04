use std::sync::Arc;
use rand::{thread_rng, Rng};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let filename = "/mnt/raid0/testfile";
    let block_size = 256 * 1024 * 1024; 
    let runtime_secs = 30; 
    let num_tasks = 32; 

    let total_bytes_written = Arc::new(Mutex::new(0u64));

    let tasks: Vec<_> = (0..num_tasks)
        .map(|i| {
            let total_bytes_written = Arc::clone(&total_bytes_written);
            let filename = filename.to_string();
            tokio::spawn(async move {

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&filename)
                    .await
                    .expect("Failed to open file");

                let start_time = tokio::time::Instant::now();
                let mut bytes_written = 0u64;
                let mut offset = i as u64 * block_size as u64; 

                while start_time.elapsed().as_secs() < runtime_secs {
                    let buffer = create_random_buffer(block_size);

                    file.seek(SeekFrom::Start(offset))
                        .await
                        .expect("Seek failed");

                    file.write_all(&buffer).await.expect("Write failed");

                    offset += block_size as u64 * num_tasks as u64;
                    bytes_written += block_size as u64;

                    let mut total = total_bytes_written.lock().await;
                    *total += block_size as u64;
                }

                let elapsed = start_time.elapsed().as_secs_f64();
                let throughput_mib = (bytes_written as f64 / (1024. * 1024.)) / elapsed;
                println!(
                    "Task {}: Written {} bytes, Throughput: {:.2} MiB/s",
                    i, bytes_written, throughput_mib
                );
            })
        })
        .collect();

    for task in tasks {
        task.await.unwrap();
    }

    let elapsed = runtime_secs as f64;
    let total_bytes = *total_bytes_written.lock().await;
    let total_throughput_mib = (total_bytes as f64 / 1_048_576.0) / elapsed;
    println!(
        "Total: Written {} bytes, Throughput: {:.2} MiB/s",
        total_bytes, total_throughput_mib
    );

    Ok(())
}

fn create_random_buffer(size: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; size];
    thread_rng().fill(&mut buffer[..]);
    buffer
}
