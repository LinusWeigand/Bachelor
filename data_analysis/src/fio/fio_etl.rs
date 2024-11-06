fn read_log_file(file_path: &str, data_type: DataType) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    let mut data = Vec::new();
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if let (Some(x_str), Some(y_str)) = (parts.get(0), parts.get(1)) {
            if let (Ok(x), Ok(mut y)) = (x_str.trim().parse::<f64>(), y_str.trim().parse::<f64>()) {
                if data_type == DataType::Latency {
                    y /= 1_000_000.;
                }
                if data_type == DataType::Throughput {
                    y /= 1024.;
                }
                data.push((x, y));
            }
        }
    }
    Ok(data)
}

#[derive(PartialEq)]
pub enum DataType {
    Latency,
    Throughput,
    IOPS,
}

pub struct MetricData {
    pub bw: Vec<(f64, f64)>,
    pub iops: Vec<(f64, f64)>,
    pub lat: Vec<(f64, f64)>,
    pub clat: Vec<(f64, f64)>,
}

impl MetricData {
    pub fn new(file_path: &str) -> MetricData {
        println!(
            "File Path: {}",
            &format!("./data/{}/bw.log", file_path)
        );

        println!(
            "Current working directory: {:?}",
            std::env::current_dir().unwrap()
        );
        let bw = read_log_file(
            &format!("./data/latency/{}/bw.log", file_path),
            DataType::Throughput,
        )
        .unwrap();
        let iops = read_log_file(
            &format!("./data/latency/{}/iops.log", file_path),
            DataType::IOPS,
        )
        .unwrap();
        let lat = read_log_file(
            &format!("./data/latency/{}/lat.log", file_path),
            DataType::Latency,
        )
        .unwrap();
        let clat = read_log_file(
            &format!("./data/latency/{}/clat.log", file_path),
            DataType::Latency,
        )
        .unwrap();

        MetricData {
            bw,
            iops,
            lat,
            clat,
        }
    }
}
