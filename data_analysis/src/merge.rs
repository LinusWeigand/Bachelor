use csv::{ReaderBuilder, Writer};
use std::{collections::HashMap, error::Error, fs::File};
fn merge_csvs() -> Result<(), Box<dyn Error>> {
    let ec2_file = File::open("ec2_data.csv")?;
    let mut ec2_reader = ReaderBuilder::new().from_reader(ec2_file);

    let ebs_file = File::open("scraped_data.csv")?;
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

    let mut writer = Writer::from_path("merged.csv")?;

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

fn main() -> Result<(), Box<dyn Error>> {
    merge_csvs()?;
    Ok(())
}
