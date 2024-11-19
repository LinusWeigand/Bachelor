use std::{
    error::Error,
    fs::File,
    io::{self, BufRead},
};

use super::utils::{FromParts, CPU, RAM, SOFTIRQ};

fn read_log_file<T>(file_path: &str) -> Result<Vec<(f64, T)>, Box<dyn Error>> where T: FromParts {
    let mut data = Vec::new();
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        let timestamp: f64 = parts[0].parse()?;
        let value = T::from_parts(&parts[1..])?;
        data.push((timestamp, value));
    }
    Ok(data)
}

fn read_net_log_file(file_path: &str) -> Result<Vec<(f64, f64)>, Box<dyn Error>> {
    let mut data = Vec::new();
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut last_bytes = 0.;
    let mut start_timestamp = 0.;
    let mut start_bytes = 0.;
    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split_ascii_whitespace().collect();

        if let (Some(timestamp), Some(bytes)) = (parts.get(0), parts.get(1)) {
            if let (Ok(mut timestamp), Ok(mut bytes)) =
                (timestamp.trim().parse::<f64>(), bytes.trim().parse::<f64>())
            {
                timestamp -= start_timestamp;
                bytes -= start_bytes;
                if start_bytes == 0. && start_timestamp == 0. {
                    start_timestamp = timestamp;
                    start_bytes = bytes;
                }
                data.push((timestamp, (bytes - last_bytes) / (1024. * 1024.)));
                last_bytes = bytes;
            }
        }
    }
    Ok(data)
}

fn read_cpu_log_file(file_path: &str) -> Result<Vec<Vec<(f64, CPU)>>, Box<dyn Error>> {
    let mut data = vec![Vec::new(); 15];
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 9 {
            if let (Ok(timestamp), Ok(core_index), Ok(cpu)) = (
                parts[0].trim().parse::<f64>(),
                parts[1][3..].trim().parse::<usize>(),
                CPU::from_parts(&parts[2..]),
            ) {
                if core_index < 15 {
                    data[core_index].push((timestamp, cpu));
                }
            }
        }
    }
    Ok(data)
}


pub struct MetricData {
    pub cpu: Vec<Vec<(f64, CPU)>>,
    pub softirq: Vec<(f64, SOFTIRQ)>,
    pub ram: Vec<(f64, RAM)>,
    pub send: Vec<(f64, f64)>,
    pub received: Vec<(f64, f64)>,
}

impl MetricData {
    pub fn new(file_path: &str) -> MetricData {
        let path = &format!("./data/{}/", file_path);
        let cpu = read_cpu_log_file(&format!("{}cpu.log", path)).unwrap();
        let softirq = read_log_file(&format!("{}softirq.log", path)).unwrap();
        let ram = read_log_file(&format!("{}ram.log", path)).unwrap();
        let send = read_net_log_file(&format!("{}send.log", path)).unwrap();
        let received = read_net_log_file(&format!("{}/receive.log", path)).unwrap();

        MetricData {
            cpu,
            softirq,
            ram,
            send,
            received,
        }
    }
}
