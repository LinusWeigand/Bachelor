use std::{
    error::Error,
    fs::File,
    io::{self, BufRead},
};

use super::{
    utils::{FromParts, CPU, RAM, SOFTIRQ},
    CORE_COUNT,
};

fn read_log_file<T>(file_path: &str) -> Result<Vec<(f64, T)>, Box<dyn Error>>
where
    T: FromParts,
{
    let mut data = Vec::new();
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines().skip(1) {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if let Err(_) = parts[0].parse::<f64>(){
            println!("timestamp could not be parsed");
        }
        
        if let Err(_) = T::from_parts(&parts[1..]){
            println!("struct could not be parsed");
        }
        let value = T::from_parts(&parts[1..])?;

        let timestamp: f64 = parts[0].parse()?;
        data.push((timestamp, value));
    }
    Ok(data)
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

fn get_ram_used(data: &Vec<(f64, RAM)>) -> Vec<(f64, f64)> {
    let start_timestamp = data[0].0;
    data.iter()
        .map(|(timestamp, ram)| (timestamp - start_timestamp, ram.total - ram.available))
        .collect()
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
        let parts: Vec<&str> = line.split_whitespace().collect();

        let mut timestamp: f64 = parts[0].parse()?;
        let mut bytes: f64 = parts[1].parse()?;
        timestamp -= start_timestamp;
        bytes -= start_bytes;

        if start_bytes == 0. && start_timestamp == 0. {
            start_timestamp = timestamp;
            start_bytes = bytes;
        }
        data.push((timestamp, (bytes - last_bytes) / (1024. * 1024. * 1024.)));
        last_bytes = bytes;
    }
    Ok(data)
}

fn read_cpu_log_file(file_path: &str) -> Result<Vec<Vec<(f64, CPU)>>, Box<dyn Error>> {
    let mut data = vec![Vec::new(); CORE_COUNT];
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
                if core_index < (CORE_COUNT - 1) {
                    data[core_index].push((timestamp, cpu));
                }
            }
        }
    }
    Ok(data)
}

fn get_cpu_util_per_core(data: &Vec<Vec<(f64, CPU)>>) -> Vec<Vec<(f64, f64)>> {
    let start_timestamp = data[0][0].0;
    data.iter()
        .map(|cores| {
            cores
                .iter()
                .map(|(timestamp, cpu)| {
                    let active_time = cpu.user + cpu.system + cpu.nice + cpu.softirq + cpu.irq;
                    let total_time = active_time + cpu.idle + cpu.iowait;
                    let utilization = match total_time {
                        t if t <= 0. => 0.,
                        _ => active_time / total_time * 100.,
                    };
                    (timestamp - start_timestamp, utilization)
                })
                .collect()
        })
        .collect()
}

struct NetworkIO {
    pub send: f64,
    pub receive: f64,
}

fn get_net_io(
    cpu_data: &Vec<Vec<(f64, f64)>>,
    softirq_data: &Vec<(f64, SOFTIRQ)>,
) -> Vec<(f64, NetworkIO)> {
    let mut utilization = vec![0.; cpu_data[0].len()];

    for core in cpu_data {
        for (i, (_, util)) in core.iter().enumerate() {
            utilization[i] += util;
        }
    }
    let start_timestamp = softirq_data[0].0;
    softirq_data
        .iter()
        .zip(utilization.iter())
        .map(|((timestamp, softirq), utilization)| {
            let timestamp = timestamp - start_timestamp;
            (
                timestamp,
                NetworkIO {
                    send: softirq.net_tx / softirq.total * utilization,
                    receive: softirq.net_tr / softirq.total * utilization,
                },
            )
        })
        .collect()
}

fn get_disk_io(cpu_data: &Vec<Vec<(f64, CPU)>>) -> Vec<(f64, f64)> {
    let mut result: Vec<(f64, f64)> = (0..cpu_data[0].len()).map(|i| (i as f64, 0.)).collect();

    for core in cpu_data {
        for (i, (_, cpu)) in core.iter().enumerate() {
            let total_time =
                cpu.user + cpu.system + cpu.nice + cpu.softirq + cpu.irq + cpu.idle + cpu.iowait;
            result[i].1 += cpu.iowait / total_time;
        }
    }
    result
}

pub struct MetricData {
    pub cpu_util: Vec<Vec<(f64, f64)>>,
    pub net_io: Vec<(f64, f64)>,
    pub send_io: Vec<(f64, f64)>,
    pub receive_io: Vec<(f64, f64)>,
    pub disk_io: Vec<(f64, f64)>,
    pub ram: Vec<(f64, f64)>,
    pub send: Vec<(f64, f64)>,
    pub received: Vec<(f64, f64)>,
}

impl MetricData {
    pub fn new(file_path: &str) -> MetricData {
        let path = &format!("./data/{}", file_path);

        let cpu_data = read_cpu_log_file(&format!("{}/cpu.log", path)).unwrap();
        let cpu_util = get_cpu_util_per_core(&cpu_data);

        let softirq_data = read_log_file(&format!("{}/softirq.log", path)).unwrap();
        let net_io_data = get_net_io(&cpu_util, &softirq_data);
        let net_io = net_io_data
            .iter()
            .map(|(t, v)| (*t, v.send + v.receive))
            .collect();
        let send_io = net_io_data.iter().map(|(t, v)| (*t, v.send)).collect();
        let receive_io = net_io_data.iter().map(|(t, v)| (*t, v.receive)).collect();

        let disk_io = get_disk_io(&cpu_data);

        // let ram_data = read_log_file(&format!("{}/ram.log", path)).unwrap();
        // let ram = get_ram_used(&ram_data);
        let ram_data = read_ram_log_file(&format!("{}/ram.log", path)).unwrap();
        let ram = get_ram_used(&ram_data);

        let send = read_net_log_file(&format!("{}/send.log", path)).unwrap();
        let received = read_net_log_file(&format!("{}/receive.log", path)).unwrap();

        MetricData {
            cpu_util,
            net_io,
            send_io,
            receive_io,
            disk_io,
            ram,
            send,
            received,
        }
    }
}
