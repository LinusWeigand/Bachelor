use std::{
    error::Error, fs::File, io::{self, BufRead}
};

#[derive(Clone)]
pub struct RAM {
    pub total: f64,
    pub used: f64,
    pub free: f64,
    pub available: f64,
}

#[derive(Clone, Debug)]
pub struct CPU {
    pub user: f64,
    pub nice: f64,
    pub system: f64,
    pub idle: f64,
    pub iowait: f64,
    pub irq: f64,
    pub softirq: f64,
}

impl CPU {
    fn from_parts(parts: &[&str]) -> Result<Self, Box<dyn Error>> {
        if parts.len() < 7 {
            return Err("Insufficient CPU parts".into());
        }

        Ok(CPU {
            user: parts[0].trim().parse::<f64>()?,
            nice: parts[1].trim().parse::<f64>()?,
            system: parts[2].trim().parse::<f64>()?,
            idle: parts[3].trim().parse::<f64>()?,
            iowait: parts[4].trim().parse::<f64>()?,
            irq: parts[5].trim().parse::<f64>()?,
            softirq: parts[6].trim().parse::<f64>()?,
        })
    }
    
}


fn read_ram_log_file(file_path: &str) -> Result<Vec<(f64, RAM)>, Box<dyn Error>> {
    let mut data = Vec::new();
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split(' ').collect();

        if let (Some(timestamp), Some(total), Some(used), Some(free), Some(available)) 
            = (parts.get(0), parts.get(1), parts.get(2), parts.get(3), parts.get(4)) {
            if let (Ok(timestamp), Ok(total), Ok(used), Ok(free), Ok(available)) 
                = (timestamp.trim().parse::<f64>(), 
                 total.trim().parse::<f64>(), 
                 used.trim().parse::<f64>(), 
                 free.trim().parse::<f64>(), 
                 available.trim().parse::<f64>()) {
                data.push((timestamp, RAM {
                    total,
                    used,
                    free,
                    available
                }));
            }
        }
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
        let parts: Vec<&str> = line.split(' ').collect();

        if let (Some(timestamp), Some(bytes)) = (parts.get(0), parts.get(1)) {
            if let (Ok(mut timestamp), Ok(mut bytes)) = (timestamp.trim().parse::<f64>(), bytes.trim().parse::<f64>()) {
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
            if let (
                Ok(timestamp),
                Ok(core_index),
                Ok(cpu),
            )
            = (
                parts[0].trim().parse::<f64>(),
                parts[1][3..].trim().parse::<usize>(),
                CPU::from_parts(&parts[2..]),
            ) {
                if core_index < 15 {
                    // println!("{} {} {:?}", timestamp, core_index, &cpu);
                    data[core_index].push((timestamp, cpu));
                }
            }
        } 
    }
    Ok(data)
}


pub struct MetricData {
    pub cpu: Vec<Vec<(f64, CPU)>>,
    pub ram: Vec<(f64, RAM)>,
    pub send: Vec<(f64, f64)>,
    pub received: Vec<(f64, f64)>,
}

impl MetricData {
    pub fn new(file_path: &str) -> MetricData {

        let path = &format!("./data/{}/", file_path);
        let cpu = read_cpu_log_file(&format!("{}cpu.log", path)).unwrap();
        let ram = read_ram_log_file(&format!("{}ram.log", path)).unwrap();
        let send = read_net_log_file(&format!("{}send.log", path)).unwrap();
        let received = read_net_log_file(&format!("{}/receive.log", path)).unwrap();

        MetricData {
            cpu,
            ram,
            send,
            received,
        }
    }
}
