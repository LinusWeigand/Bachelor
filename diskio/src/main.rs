use std::alloc;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let filename = "/mnt/raid0/testfile";
    let block_size = 256 * 1024 * 1024; // 256 MB
    let runtime_secs = 30; // Runtime in seconds
    let num_tasks = 32; // Simulate iodepth=32
    let alignment = 512; // Alignment for compatibility

    // Create aligned buffer
    let buffer = Arc::new(create_aligned_buffer(block_size, alignment));

    // Total bytes written by all tasks (shared across tasks)
    let total_bytes_written = Arc::new(Mutex::new(0u64));

    // Spawn tasks
    let tasks: Vec<_> = (0..num_tasks)
        .map(|i| {
            let buffer = Arc::clone(&buffer);
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
                let mut offset = i as u64 * block_size as u64; // Unique offset for each task

                while start_time.elapsed().as_secs() < runtime_secs {
                    // Seek to the correct position
                    file.seek(SeekFrom::Start(offset))
                        .await
                        .expect("Seek failed");

                    // Write aligned buffer
                    file.write_all(&buffer)
                        .await
                        .expect("Write failed");

                    // Update offset and bytes written
                    offset += block_size as u64 * num_tasks as u64;
                    bytes_written += block_size as u64;

                    // Update total bytes written (thread-safe)
                    let mut total = total_bytes_written.lock().await;
                    *total += block_size as u64;
                }

                let elapsed = start_time.elapsed().as_secs_f64();
                let throughput_mib = (bytes_written as f64 / 1_048_576.0) / elapsed;
                println!(
                    "Task {}: Written {} bytes, Throughput: {:.2} MiB/s",
                    i, bytes_written, throughput_mib
                );
            })
        })
        .collect();

    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }

    // Calculate total throughput
    let elapsed = runtime_secs as f64;
    let total_bytes = *total_bytes_written.lock().await;
    let total_throughput_mib = (total_bytes as f64 / 1_048_576.0) / elapsed;
    println!(
        "Total: Written {} bytes, Throughput: {:.2} MiB/s",
        total_bytes, total_throughput_mib
    );

    Ok(())
}

/// Creates an aligned buffer for direct I/O.
fn create_aligned_buffer(size: usize, alignment: usize) -> Vec<u8> {
    let layout = alloc::Layout::from_size_align(size, alignment).unwrap();
    unsafe {
        let ptr = alloc::alloc_zeroed(layout);
        Vec::from_raw_parts(ptr, size, size)
    }
}
