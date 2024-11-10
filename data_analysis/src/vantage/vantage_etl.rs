use std::{
    error::Error,
    fs::File,
    io::{self, BufRead},
};

use csv::ReaderBuilder;
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

    #[serde(rename = "Linux Reserved cost")]
    linux_reserved_cost: String,

    #[serde(rename = "EBS Optimized: Baseline Throughput (128K)")]
    baseline_throughput: String,
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

fn parse_throughput(throughput: &str) -> Option<f64> {
    let parts: Vec<&str> = throughput.split_whitespace().collect();

    return match parts.len() {
        2 => {
            return parts[0].parse::<f64>().ok();
        }
        _ => None,
    };
}

fn read_csv(file_path: &str) -> Result<Vec<EC2Instance>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b',')
        .flexible(true)
        .from_path(file_path)?;
    let mut instances: Vec<EC2Instance> = Vec::new();

    for result in reader.deserialize() {
        let record: EC2Instance = result?;
        instances.push(record);
    }

    Ok(instances)
}

pub struct InstanceData {
    instances: Vec<EC2Instance>,
}

impl InstanceData {
    pub fn new() -> InstanceData {
        let instances = read_csv("./frankfurt.csv").unwrap();
        InstanceData { instances }
    }

    pub fn get_cost_throughput(&self) -> Vec<(String, f64, f64)> {
        let mut data_points: Vec<(String, f64, f64)> = Vec::new();
        for instance in &self.instances {
            let price = match parse_price(&instance.linux_reserved_cost) {
                None => continue,
                Some(v) => v,
            };
            let baseline_throughput = match parse_throughput(&instance.baseline_throughput) {
                None => continue,
                Some(v) => v,
            };
            data_points.push((instance.api_name.clone(), price, baseline_throughput));
        }
        data_points
    }

    pub fn get_cost_per_gb(&self) -> Vec<(String, f64)> {
        let mut data_points: Vec<(String, f64)> = Vec::new();
        for instance in &self.instances {
            let price = match parse_price(&instance.linux_reserved_cost) {
                None => continue,
                Some(v) => v,
            };

            let storage = match parse_storage(&instance.storage) {
                None => continue,
                Some(v) => v,
            };

            // if (price / storage.0 as f64) > 100. {
            //     continue;
            // }
            data_points.push((
                format!("{} {}", instance.api_name.clone(), storage.1),
                price / (storage.0 as f64),
            ));
        }
        data_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        data_points
    }

    pub fn get_network_performance_per_gb(&self) -> Vec<(String, f64)> {
        let mut data_points: Vec<(String, f64)> = Vec::new();
        for instance in &self.instances {
            if !instance.api_name.trim().starts_with("d3en") {
                continue;
            }
            let network_performance = match parse_network_performance(&instance.network_performance)
            {
                None => continue,
                Some(v) => v,
            };

            let storage = match parse_storage(&instance.storage) {
                None => continue,
                Some(v) => v,
            };

            // if (price / storage.0 as f64) > 100. {
            //     continue;
            // }
            data_points.push((
                format!("{} {}", instance.api_name.clone(), storage.1),
                network_performance / (storage.0 as f64),
            ));
        }
        data_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        data_points
    }

    pub fn get_cost_per_throughput(&self) -> Vec<(String, f64)> {
        let mut data_points: Vec<(String, f64)> = Vec::new();
        for instance in &self.instances {
            let price = match parse_price(&instance.linux_reserved_cost) {
                None => continue,
                Some(v) => v,
            };
            let baseline_throughput = match parse_throughput(&instance.baseline_throughput) {
                None => continue,
                Some(v) => v,
            };
            data_points.push((instance.api_name.clone(), baseline_throughput / price));
        }
        data_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        data_points
    }

    pub fn get_efficient_frontier(&self) -> Vec<(String, f64, f64)> {
        let mut combinations = Vec::new();
        let storage = 100_000;
        for instance in &self.instances {
            let baseline_throughput = match parse_throughput(&instance.baseline_throughput) {
                None => continue,
                Some(v) => v,
            };
            let price = match parse_price(&instance.linux_reserved_cost) {
                None => continue,
                Some(v) => v,
            };

            let storage_per_instance = match parse_storage(&instance.storage) {
                None => 0,
                Some((v, _)) => v,
            };

            for i in (125..16_000).step_by(1000) {
                let initial_disk_storage = 100_000;
                let mut disk_count = (storage as f64 / i as f64).floor();
                let mut min_instances = (disk_count / 40.).ceil() as u32;
                let mut instance_storage = storage_per_instance * min_instances;
                let mut remaining_disk_storage;
                if instance_storage > initial_disk_storage {
                    remaining_disk_storage = 0;
                } else {
                    remaining_disk_storage = initial_disk_storage - instance_storage;
                }

                for i in 1..10 {
                    if storage_per_instance == 0 || remaining_disk_storage == 0 {
                        continue;
                    }

                    disk_count = (remaining_disk_storage as f64 / i as f64).floor();
                    let next_instance_count = (disk_count / 40.).ceil() as u32;
                    if next_instance_count == min_instances {
                        continue;
                    }
                    min_instances = next_instance_count;
                    instance_storage = storage_per_instance * min_instances;
                    if instance_storage > initial_disk_storage {
                        remaining_disk_storage = 0;
                        continue;
                    } else {
                        remaining_disk_storage = initial_disk_storage - instance_storage;
                    }
                }

                let total_cost_per_month = remaining_disk_storage as f64 * 0.01596
                    + min_instances as f64 * price
                    + min_instances as f64 * 8. * 0.04575; //Root St1 Volume

                // if total_cost_per_month > 2250. {
                //     continue;
                // }

                let max_throughput_per_disk = 250.;
                let disk_throughput: f64 = max_throughput_per_disk * disk_count as f64;
                let instance_throughput: f64 = baseline_throughput * min_instances as f64;
                let throughput: f64 = f64::min(disk_throughput, instance_throughput);
                if throughput.is_nan() || throughput.is_infinite() {
                    continue;
                }
                combinations.push((
                    format!(
                        "{} sc1 disks, {} {}",
                        disk_count, min_instances, &instance.name
                    ),
                    total_cost_per_month,
                    throughput,
                ));
            }
        }
        combinations
    }
}
