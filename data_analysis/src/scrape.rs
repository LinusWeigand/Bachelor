use csv::Writer;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ebs-optimized.html";

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .build()?;

    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch URL: Status {}", response.status()).into());
    }

    let body = response.text()?;

    let document = Html::parse_document(&body);

    let row_selector = Selector::parse("tr").unwrap();

    let file = File::create("scraped_data.csv")?;
    let mut wtr = Writer::from_writer(file);

    wtr.write_record(&[
        "Instance size",
        "Baseline bandwidth (Mbps)",
        "Maximum bandwidth (Mbps)",
        "Baseline throughput (MB/s, 128 KiB I/O)",
        "Maximum throughput (MB/s, 128 KiB I/O)",
        "Baseline IOPS (16 KiB I/O)",
        "Maximum IOPS (16 KiB I/O)",
    ])?;

    for row in document.select(&row_selector) {
        let data_selector = Selector::parse("td").unwrap();
        let mut row_data = Vec::new();

        for cell in row.select(&data_selector) {
            let text = cell.text().collect::<Vec<_>>().concat();
            row_data.push(text);
        }

        if row_data.len() > 0 {
            row_data[0] = row_data[0]
                .strip_suffix('1')
                .unwrap_or(row_data[0].as_str())
                .trim()
                .to_string();
        }

        if row_data.len() == 7 {
            wtr.write_record(&row_data)?;
        }

        if row_data.len() == 4 {
            let mut new_row_data = Vec::new();
            new_row_data.push(&row_data[0]);
            new_row_data.push(&row_data[1]);
            new_row_data.push(&row_data[1]);
            new_row_data.push(&row_data[2]);
            new_row_data.push(&row_data[2]);
            new_row_data.push(&row_data[3]);
            new_row_data.push(&row_data[3]);
        }
    }

    wtr.flush()?;

    println!("Data successfully written to aws_ebs_data.csv");

    Ok(())
}
