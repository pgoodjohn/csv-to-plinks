// main.rs
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use csv::ReaderBuilder;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio;

fn main() {
    plogger::init(true);

    let records = match read_csv("data.csv") {
        Ok(records) => records,
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    // Process the records, e.g. interact with the Mollie API
    let api_key = "MOLLIE_API_KEY";

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut payment_infos = Vec::new();
    rt.block_on(async {
        for record in &records {
            match create_payment_request(api_key, record).await {
                Ok(payment_info) => payment_infos.push(payment_info),
                Err(err) => println!("Error: {}", err),
            }
        }
    });

    write_payment_infos_to_csv("payment_links.csv", &payment_infos).unwrap();
}

// Define a structure to hold the data
#[derive(Debug, Deserialize)]
struct Record {
    name: String,
    amount_owed: f64,
    item_ordered: String,
}

#[derive(Debug, Serialize)]
struct PaymentRequest {
    amount: Amount,
    description: String,
    #[serde(rename = "redirectUrl")]
    redirect_url: String,
}

#[derive(Debug, Serialize)]
struct Amount {
    currency: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct PaymentLinkResponse {
    _links: PaymentLinks,
}

#[derive(Debug, Deserialize)]
struct PaymentLinks {
    #[serde(rename = "paymentLink")]
    payment_link: PaymentLink,
}

#[derive(Debug, Deserialize)]
struct PaymentLink {
    href: String,
}

#[derive(Debug)]
struct PaymentInfo {
    payment_link: String,
    name: String,
    amount: String,
}

async fn create_payment_request(
    api_key: &str,
    record: &Record,
) -> Result<PaymentInfo, Box<dyn Error>> {
    let client = Client::new();
    let payment_request = PaymentRequest {
        amount: Amount {
            currency: "EUR".to_string(),
            value: format!("{:.2}", record.amount_owed),
        },
        description: format!("{} - {}", record.name, record.item_ordered),
        redirect_url: "https://localhost".to_string(),
    };

    let res = client
        .post("https://api.mollie.com/v2/payment-links")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payment_request)
        .send()
        .await?;

    if res.status().is_success() {
        let payment_response: PaymentLinkResponse = res.json().await?;
        println!(
            "Payment request for {} was successful. Payment link: {}",
            record.name, payment_response._links.payment_link.href
        );
        Ok(PaymentInfo {
            name: record.name.clone(),
            amount: format!("{:.2}", record.amount_owed),
            payment_link: payment_response._links.payment_link.href,
        })
    } else {
        Err(format!(
            "Payment request for {} failed: {}",
            record.name,
            res.text().await?
        )
        .into())
    }
}

fn read_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut buffer = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buffer)?;

    let mut rdr = ReaderBuilder::new().from_reader(&buffer[..]);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn write_payment_infos_to_csv<P: AsRef<Path>>(
    path: P,
    payment_infos: &[PaymentInfo],
) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(path)?;

    wtr.write_record(&["Name", "Amount", "Payment Link"])?;

    for payment_info in payment_infos {
        wtr.write_record(&[
            &payment_info.name,
            &payment_info.amount,
            &payment_info.payment_link,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}
