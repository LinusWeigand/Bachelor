use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{download::Range, get::GetObjectRequest},
};
use std::io::Write;
use std::{
    collections::HashMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Cursor},
    process::exit,
};

const THESIOS_BUCKET_OBJECTS_PATH: &str = "./data/thesios_bucket_objects";
const ESTIMATED_FILE_SIZES_PATH: &str = "./data/estimated_file_sizes.csv";
const SELECTIVITY_WRITE_PATH: &str = "./data/selectivity_write";
const SELECTIVITY_READ_PATH: &str = "./data/selectivity_read";
const BUCKET_NAME: &str = "thesios-io-traces";

const ITERATIONS: u32 = 10;

struct AggregatedData {
    estimated_size: u64,
    read_bytes: u64,
    write_bytes: u64,
}

pub struct Latency {
    pub latency: f64,
    pub count: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config);

    for _ in 0..ITERATIONS {
        next_iteration(&client).await?;
    }
    calculate_selectivity()?;

    Ok(())
}

async fn next_iteration(client: &Client) -> Result<(), Box<dyn Error>> {
    let mut estimated_file_sizes = load_estimated_sizes(ESTIMATED_FILE_SIZES_PATH)?;
    let mut write_selectivity_map = load_selectivity_map(SELECTIVITY_WRITE_PATH)?;
    let mut read_selectivity_map = load_selectivity_map(SELECTIVITY_READ_PATH)?;

    let mut object_name = "".to_string();

    let file = File::open(THESIOS_BUCKET_OBJECTS_PATH)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        let progress = parts[1];
        if progress == "todo" {
            object_name = parts[0].to_string();
        } else {
            continue;
        }
    }

    if object_name == "" {
        println!("All objects done");
        exit(0);
    }

    println!("{}", object_name);
    let result = client
        .download_object(
            &GetObjectRequest {
                bucket: BUCKET_NAME.to_string(),
                object: object_name.to_string(),
                ..Default::default()
            },
            &Range::default(),
        )
        .await?;

    println!("downloaded file");

    let cursor = Cursor::new(result);

    let reader = std::io::BufReader::new(cursor);

    for line in reader.lines().skip(1) {
        let line = line?;

        let parts: Vec<&str> = line.split(',').collect();
        let filename = parts[0].to_string();
        let offset = parts[1].parse::<u64>()?;
        // let application = parts[2].to_string();
        let op_type = parts[6].to_string();
        let from_flash_cache = parts[8].parse::<i8>()?;
        let cache_hit = parts[9].parse::<i8>()?;
        let request_io_size_bytes = parts[10].parse::<u64>()?;
        let disk_io_size_bytes = parts[11].parse::<u64>()?;
        let response_io_size_bytes = parts[12].parse::<u64>()?;
        let latency = parts[16].parse::<f64>()?;

        if from_flash_cache == 1 || cache_hit == 1 {
            continue;
        }

        let mut read_bytes = 0;
        let mut write_bytes = 0;
        let estimated_size = if op_type == "WRITE" {
            write_bytes = std::cmp::max(request_io_size_bytes, disk_io_size_bytes);
            offset + request_io_size_bytes
        } else if op_type == "READ" {
            read_bytes = std::cmp::max(response_io_size_bytes, disk_io_size_bytes);
            offset + response_io_size_bytes
        } else {
            continue;
        };

        let mut final_estimation = estimated_size;

        estimated_file_sizes
            .entry(filename)
            .and_modify(|data| {
                final_estimation = std::cmp::max(data.estimated_size, estimated_size);
                if estimated_size > data.estimated_size {
                    data.estimated_size = estimated_size;
                    data.read_bytes += read_bytes;
                    data.write_bytes += write_bytes;
                }
            })
            .or_insert(AggregatedData {
                estimated_size,
                read_bytes,
                write_bytes,
            });

        let selectivity = (disk_io_size_bytes as f64 / final_estimation as f64) as u32;

        match op_type.as_str() {
            "READ" => {
                if read_bytes > 0 {
                    let latency_per_gb = latency * ((1024. * 1024. * 1024.) / read_bytes as f64);
                    read_selectivity_map
                        .entry(selectivity)
                        .and_modify(|data| {
                            data.latency += latency_per_gb;
                            data.count += 1;
                        })
                        .or_insert(Latency {
                            latency: latency_per_gb,
                            count: 1,
                        });
                }
            }
            "WRITE" => {
                if write_bytes > 0 {
                    let latency_per_gb = latency * ((1024. * 1024. * 1024.) / write_bytes as f64);
                    write_selectivity_map
                        .entry(selectivity)
                        .and_modify(|data| {
                            data.latency += latency_per_gb;
                            data.count += 1;
                        })
                        .or_insert(Latency {
                            latency: latency_per_gb,
                            count: 1,
                        });
                }
            }
            _ => {}
        };
    }

    write_estimated_sizes(ESTIMATED_FILE_SIZES_PATH, &estimated_file_sizes)?;
    mark_object_as_done(THESIOS_BUCKET_OBJECTS_PATH, &object_name)?;
    write_down_selectivity_map(SELECTIVITY_WRITE_PATH, &write_selectivity_map)?;
    write_down_selectivity_map(SELECTIVITY_READ_PATH, &read_selectivity_map)?;

    Ok(())
}

fn load_estimated_sizes(
    file_path: &str,
) -> Result<HashMap<String, AggregatedData>, Box<dyn Error>> {
    let mut estimated_sizes = HashMap::new();

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        let filename = parts[0].to_string();
        let estimated_size = parts[1].parse::<u64>()?;
        let read_bytes = parts[2].parse::<u64>()?;
        let write_bytes = parts[2].parse::<u64>()?;

        estimated_sizes.insert(
            filename,
            AggregatedData {
                estimated_size,
                read_bytes,
                write_bytes,
            },
        );
    }

    Ok(estimated_sizes)
}

fn write_estimated_sizes(
    file_path: &str,
    estimated_sizes: &HashMap<String, AggregatedData>,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);
    for (filename, data) in estimated_sizes {
        writeln!(
            writer,
            "{} {} {} {}",
            filename, data.estimated_size, data.read_bytes, data.write_bytes,
        )?;
    }
    Ok(())
}

pub fn load_selectivity_map(file_path: &str) -> Result<HashMap<u32, Latency>, Box<dyn Error>> {
    let mut latency_map = HashMap::new();

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        let selectivity = parts[0].parse::<u32>()?;
        let latency = parts[1].parse::<f64>()?;
        let count = parts[2].parse::<u32>()?;

        latency_map.insert(selectivity, Latency { latency, count });
    }

    Ok(latency_map)
}

fn write_down_selectivity_map(
    file_path: &str,
    selectiviy_map: &HashMap<u32, Latency>,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);
    for (selectivity, data) in selectiviy_map {
        writeln!(writer, "{} {} {}", selectivity, data.latency, data.count,)?;
    }
    Ok(())
}

fn mark_object_as_done(file_path: &str, object_name: &str) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.contains(object_name) {
            lines.push(format!("{} done", object_name));
        } else {
            lines.push(line);
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

fn calculate_selectivity() -> Result<(), Box<dyn Error>> {
    let file = File::open(ESTIMATED_FILE_SIZES_PATH)?;
    let reader = BufReader::new(file);

    let mut total_read_bytes: u128 = 0;
    let mut total_write_bytes: u128 = 0;
    let mut total_bytes: u128 = 0;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        let size = parts[1].parse::<u128>()?;
        let read_bytes = parts[2].parse::<u128>()?;
        let write_bytes = parts[3].parse::<u128>()?;

        total_bytes += size;
        total_read_bytes += read_bytes;
        total_write_bytes += write_bytes;
    }
    let read_selectivity: f64 = total_read_bytes as f64 / total_bytes as f64;
    let write_selectivity: f64 = total_write_bytes as f64 / total_bytes as f64;
    println!("Read Selectivity: {}", read_selectivity);
    println!("Write Selectivity: {}", write_selectivity);
    Ok(())
}
