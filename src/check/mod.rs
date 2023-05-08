use clap::Parser;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use csv::ReaderBuilder;
use reqwest::Client;
use serde_derive::Deserialize;
use tokio;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Parser, Debug)]
#[clap(arg_required_else_help(true))]
pub struct CheckCommand {
    #[clap(short, long, global = true)]
    debug: bool,

    #[clap(short, long)]
    input: String,

    #[clap(long)]
    api_key: String,
}

pub fn command(command: &CheckCommand) -> Result<String, Box<dyn std::error::Error>> {
    log::debug!("Running csv-to-plinks: {:?}", command);

    match validate_file_paths(&command.input) {
        Ok(()) => {}
        Err(err) => {
            log::error!("Error: {}", err);
            std::process::exit(-1);
        }
    }

    let records = match read_csv(&command.input) {
        Ok(records) => records,
        Err(err) => {
            println!("Error: {}", err);
            panic!("No input file");
        }
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        for record in &records {
            match get_payment_request_status(&command.api_key, &record.payment_link).await {
                Ok(payment_request_status) => {
                    log::info!(
                        "Payment Request for {} from {} (id: {}) was paid at: {}",
                        record.name,
                        record.amount,
                        payment_request_status.id,
                        match payment_request_status.paid_at {
                            Some(t) => t,
                            None => "N/A".to_string(),
                        }
                    );
                }
                Err(err) => println!("Error: {}", err),
            }
        }
    });

    Ok("Ok".to_string())
}

#[derive(Deserialize, Debug)]
struct PaymentInfo {
    #[serde(rename = "Payment Link")]
    payment_link: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Amount")]
    amount: String,
}

#[derive(Debug, Deserialize)]
struct PaymentRequestStatus {
    id: String,
    #[serde(rename = "paidAt")]
    paid_at: Option<String>,
}

lazy_static! {
    static ref PAYMENT_LINK_RE: Regex =
        Regex::new(r"https://paymentlink\.mollie\.com/payment/([\w-]+)/").unwrap();
}

async fn get_payment_request_status(
    api_key: &str,
    payment_link: &str,
) -> Result<PaymentRequestStatus, Box<dyn Error>> {
    let payment_request_token = PAYMENT_LINK_RE
        .captures(payment_link)
        .ok_or("Failed to extract payment request token from the payment link")?
        .get(1)
        .map(|m| m.as_str())
        .ok_or("Failed to extract payment request token from the payment link")?;

    let client = Client::new();
    let res = client
        .get(&format!(
            "https://api.mollie.com/v2/payment-links/pl_{}",
            payment_request_token
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if res.status().is_success() {
        let payment_request_status: PaymentRequestStatus = res.json().await?;
        Ok(payment_request_status)
    } else {
        Err(format!(
            "Failed to get payment request status: {}",
            res.text().await?
        )
        .into())
    }
}

fn validate_file_paths(input: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(input);

    if !input_path.exists() {
        return Err(format!("Input file does not exist: {}", input).into());
    }

    Ok(())
}

fn read_csv<P: AsRef<Path>>(path: P) -> Result<Vec<PaymentInfo>, Box<dyn Error>> {
    let mut buffer = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buffer)?;

    let mut rdr = ReaderBuilder::new().from_reader(&buffer[..]);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: PaymentInfo = result?;
        records.push(record);
    }

    Ok(records)
}
