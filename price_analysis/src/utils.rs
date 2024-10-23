use std::{error::Error, fs::File};

use csv::{ReaderBuilder, Writer};
use plotly::{
    common::{HoverInfo, Marker, Mode},
    Plot, Scatter,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EC2Instance {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "API Name")]
    api_name: String,

    #[serde(rename = "Instance Memory")]
    memory: String,

    #[serde(rename = "vCPUs")]
    vcpus: String,

    #[serde(rename = "Instance Storage")]
    storage: String,

    #[serde(rename = "Network Performance")]
    network_performance: String,

    #[serde(rename = "On Demand")]
    on_demand: String,

    #[serde(rename = "Linux Reserved cost")]
    linux_reserved_cost: String,

    #[serde(rename = "Linux Spot Minimum cost")]
    linux_spot_min_cost: String,

    #[serde(rename = "Windows On Demand cost")]
    windows_on_demand: String,

    #[serde(rename = "Windows Reserved cost")]
    windows_reserved_cost: String,

    #[serde(rename = "Baseline bandwidth (Mbps)")]
    baseline_bandwidth: Option<f64>,

    #[serde(rename = "Maximum bandwidth (Mbps)")]
    maximum_bandwidth: Option<f64>,

    #[serde(rename = "Baseline throughput (MB/s, 128 KiB I/O)")]
    baseline_throughput: Option<f64>,

    #[serde(rename = "Maximum throughput (MB/s, 128 KiB I/O)")]
    maximum_throughput: Option<f64>,

    #[serde(rename = "Baseline IOPS (16 KiB I/O)")]
    baseline_iops: Option<f64>,

    #[serde(rename = "Maximum IOPS (16 KiB I/O)")]
    maximum_iops: Option<f64>,
}

fn parse_vcpus(vcpus: &str) -> Option<(u32, Option<String>)> {
    let parts: Vec<&str> = vcpus.split_whitespace().collect();

    if let Ok(vcpu_count) = parts[0].parse::<u32>() {
        let burst = if vcpus.contains("burst") {
            Some(parts[4..].join(" "))
        } else {
            None
        };
        return Some((vcpu_count, burst));
    }
    None
}

fn parse_memory(memory: &str) -> Option<f64> {
    if let Some(value) = memory.split_whitespace().next() {
        return value.parse::<f64>().ok();
    }
    None
}

fn parse_network_performance(network_performance: &str) -> Option<f64> {
    let parts: Vec<&str> = network_performance.split_whitespace().collect();

    return match parts.len() {
        0 => None,
        1 => match parts[0] {
            "High" => Some(25.),     // bis zu 10?
            "Moderate" => Some(10.), // bis zu 5?
            "Low" => Some(1.),       // bis zu 0.5?
            _ => None,
        },
        2 => match parts[1] {
            "Low" => Some(0.1), // bis zu 0.25?
            "Gigabit" => parts[0].parse::<f64>().ok(),
            _ => None,
        },
        3 => {
            let multiplier = parts[0].strip_suffix('x').unwrap_or(parts[0]);
            let multiplier = multiplier.parse::<f64>().unwrap_or(-1.);
            let value = parts[1].parse::<f64>().unwrap_or(-1.);
            if value < 0. || multiplier < 0. {
                return None;
            }
            return Some(value * multiplier);
        }
        4 => parts[2].parse::<f64>().ok(),
        _ => None,
    };
}

fn parse_storage(storage: &str) -> Option<(u32, &str)> {
    let parts: Vec<&str> = storage.split_whitespace().collect();

    return match parts.len() {
        3 => {
            let value = match parts[0].parse::<u32>() {
                Err(_) => return None,
                Ok(v) => v,
            };
            let device = match parts.last() {
                None => return None,
                Some(v) => v,
            };

            Some((value, device))
        }
        4 => {
            let value = match parts[0].parse::<u32>() {
                Err(_) => return None,
                Ok(v) => v,
            };
            Some((value, "NVMe"))
        }
        7 => {
            let value = match parts[0].parse::<u32>() {
                Err(_) => return None,
                Ok(v) => v,
            };
            let device = match parts.last() {
                None => return None,
                Some(v) => v.strip_suffix(')').unwrap_or(v),
            };
            Some((value, device))
        }
        8 => {
            let value = match parts[0].parse::<u32>() {
                Err(_) => return None,
                Ok(v) => v,
            };
            Some((value, "NVMe"))
        }
        _ => None,
    };
}

fn parse_price(on_demand: &str) -> Option<f64> {
    let parts: Vec<&str> = on_demand.split_whitespace().collect();

    return match parts.len() {
        2 => {
            if let Some(value) = parts[0].strip_prefix('$') {
                return value.parse::<f64>().ok();
            }
            None
        }
        _ => None,
    };
}

fn read_csv(file_path: &str) -> Result<Vec<EC2Instance>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().delimiter(b',').from_path(file_path)?;
    let mut instances: Vec<EC2Instance> = Vec::new();

    for result in reader.deserialize() {
        let record: EC2Instance = result?;
        instances.push(record);
    }

    Ok(instances)
}

pub struct Data {
    instances: Vec<EC2Instance>,
}

impl Data {
    pub fn new() -> Data {
        let instances = read_csv("./merged.csv").unwrap();
        Data { instances }
    }

    pub fn get_cost_throughput(&self) -> Vec<(String, f64, f64)> {
        let mut data_points: Vec<(String, f64, f64)> = Vec::new();
        for instance in &self.instances {
            let price = match parse_price(&instance.linux_reserved_cost) {
                None => continue,
                Some(v) => v,
            };
            let baseline_throughput = match instance.baseline_throughput {
                None => continue,
                Some(v) => v,
            };
            data_points.push((instance.api_name.clone(), price, baseline_throughput));
        }
        data_points
    }
}