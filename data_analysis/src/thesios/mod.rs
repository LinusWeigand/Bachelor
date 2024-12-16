use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{download::Range, get::GetObjectRequest},
};
use std::{io::Write};
use std::{
    collections::HashMap,
    error::Error,
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Cursor},
    process::exit,
};

const THESIOS_BUCKET_OBJECTS_PATH: &str = "./data/thesios_bucket_objects";
const ESTIMATED_FILE_SIZES_PATH: &str = "./data/estimated_file_sizes.csv";

struct AggregatedData {
    estimated_size: u64,
    read_bytes: u64,
    write_bytes: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config);

    let bucket_name = "thesios-io-traces";

    let mut estimated_file_sizes = load_estimated_sizes(ESTIMATED_FILE_SIZES_PATH)?;

    // let objects = client
    //     .list_objects(&ListObjectsRequest {
    //         bucket: bucket_name.to_string(),
    //         ..Default::default()
    //     })
    //     .await?;

    // if let Some(items) = objects.items {
    //     let file = fs::File::create("./data/thesios_bucket_objects")?;
    //     let mut writer = BufWriter::new(file);
    //     for object in items {
    //         writeln!(writer, "{} todo", object.name)?;
    //     }
    // }

    let mut object_name = "".to_string();

    //Find nect object_name
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
                bucket: bucket_name.to_string(),
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
        // let from_flash_cache = parts[8].parse::<i8>()?;
        // let cache_hit = parts[9].parse::<i8>()?;
        let request_io_size_bytes = parts[10].parse::<u64>()?;
        let disk_io_size_bytes = parts[11].parse::<u64>()?;
        let response_io_size_bytes = parts[12].parse::<u64>()?;
        // let latency = parts[16].parse::<f64>()?;

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


        estimated_file_sizes
            .entry(filename)
            .and_modify(|data| {
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
    }

    write_estimated_sizes(ESTIMATED_FILE_SIZES_PATH, &estimated_file_sizes)?;
    mark_object_as_done(THESIOS_BUCKET_OBJECTS_PATH, &object_name)?;

    Ok(())
}

fn load_estimated_sizes(file_path: &str) -> Result<HashMap<String, AggregatedData>, Box<dyn Error>> {
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

        estimated_sizes.insert(filename, AggregatedData {
            estimated_size,
            read_bytes,
            write_bytes,
        });
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
        writeln!(writer, "{} {} {} {}", 
            filename, 
            data.estimated_size, 
            data.read_bytes, 
            data.write_bytes, 
        )?;
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
