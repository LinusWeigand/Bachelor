use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let filename = "/mnt/raid0/testfile";
    let block_size = 256 * 1024 * 1024; // 256 MB
    let runtime_secs = 30;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .custom_flags(libc::O_DIRECT) // Bypassing cache
        .open(filename)?;

    let buffer = create_aligned_buffer(block_size);

    let start_time = Instant::now();
    let mut bytes_written = 0;

    while start_time.elapsed().as_secs() < runtime_secs {
        let written = file.write(&buffer)?;
        bytes_written += written;
    }

    let elapsed = start_time.elapsed();
    let throughput_mib = (bytes_written as f64 / 1_048_576.0) / elapsed.as_secs_f64();
    println!(
        "Completed. Total written: {} bytes, Throughput: {:.2} MiB/s",
        bytes_written, throughput_mib
    );

    Ok(())
}

fn create_aligned_buffer(size: usize) -> Vec<u8> {
    //512-byte boundary for O_DIRECT compatibility
    let alignment = 512;
    let layout = std::alloc::Layout::from_size_align(size, alignment).unwrap();
    unsafe {
        let ptr = std::alloc::alloc_zeroed(layout);
        Vec::from_raw_parts(ptr, size, size)
    }
}
