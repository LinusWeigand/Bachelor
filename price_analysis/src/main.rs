use std::{collections::HashMap, error::Error, fs::File};

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

fn merge_csvs() -> Result<(), Box<dyn Error>> {
    let ec2_file = File::open("ec2.csv")?;
    let mut ec2_reader = ReaderBuilder::new().from_reader(ec2_file);

    let ebs_file = File::open("aws_ebs_data.csv")?;
    let mut ebs_reader = ReaderBuilder::new().from_reader(ebs_file);

    let mut ebs_data: HashMap<String, Vec<String>> = HashMap::new();

    let ebs_headers = ebs_reader.headers()?.clone();

    for result in ebs_reader.records() {
        let record = result?;
        let instance_size = record.get(0).unwrap().to_string();
        ebs_data.insert(
            instance_size,
            record.iter().skip(1).map(|s| s.to_string()).collect(),
        );
    }

    let mut writer = Writer::from_path("combined_output.csv")?;

    let ec2_headers = ec2_reader.headers()?.clone();
    let mut combined_headers = ec2_headers.clone();

    combined_headers.extend(ebs_headers.iter().skip(1));
    writer.write_record(&combined_headers)?;

    for result in ec2_reader.records() {
        let mut ec2_record = result?;
        let api_name = ec2_record.get(1).unwrap();

        if let Some(ebs_row) = ebs_data.get(api_name) {
            ec2_record.extend(ebs_row);
        } else {
            ec2_record.extend(vec![""; ebs_headers.len() - 1]);
        }

        writer.write_record(&ec2_record)?;
    }

    writer.flush()?;

    println!("Data successfully combined and written to combined_output.csv");

    Ok(())
}

fn plot_storage_per_dollar(instances: &[EC2Instance]) {
    let mut plot = Plot::new();

    let mut x_values: Vec<f64> = Vec::new();
    let mut y_values: Vec<f64> = Vec::new();
    let mut hover_texts: Vec<String> = Vec::new();
    let mut colors: Vec<&str> = Vec::new();

    for instance in instances {
        let storage = match parse_storage(&instance.storage) {
            None => continue,
            Some(v) => v,
        };
        let price = match parse_price(&instance.linux_reserved_cost) {
            None => continue,
            Some(v) => v,
        };
        let price_per_gb = price / (storage.0 as f64);

        if price_per_gb > 0.0008 {
            continue;
        }

        x_values.push(storage.0 as f64);

        println!("Storage: {}", storage.0);
        y_values.push(price_per_gb);
        hover_texts.push(format!(
            "{}, {}, {:.8} ",
            storage.1, &instance.name, price_per_gb
        ));
        let color = match storage.1 {
            "HDD" => "blue",
            "SSD" => "green",
            "NVMe" => "red",
            _ => "gray",
        };
        colors.push(color);
    }

    let scatter = Scatter::new(x_values, y_values)
        .hover_info(HoverInfo::Text)
        .hover_text_array(hover_texts)
        .name("EC2 Instances")
        .mode(Mode::Markers)
        .marker(Marker::new().color_array(colors));

    plot.add_trace(scatter);
    plot.show();
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

fn main() -> Result<(), Box<dyn Error>> {
    // let data = read_csv("./ec2.csv").unwrap();
    // plot_storage_per_dollar(data.as_slice());
    // let _ = plot_storage_per_dollar(data.as_slice());
    // let _ = plot_data(data.as_slice());
    merge_csvs()?;
    Ok(())
}
