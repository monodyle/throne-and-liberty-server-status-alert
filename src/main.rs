use chrono::Utc;
use reqwest::Client;
use select::document::Document;
use select::predicate::Class;
use serde_json::json;
use std::error::Error;
use std::{env, fs};
use tokio::time::{sleep, Duration};

const URL: &str = "https://www.playthroneandliberty.com/en-us/support/server-status";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    loop {
        let response = client.get(URL).send().await?.text().await?;
        let document = Document::from(response.as_str());

        let is_maintenance = is_maintenance(&document);
        let previous_status = read_previous_status()?;

        if is_maintenance != previous_status {
            send_alert(is_maintenance).await?;
            update_previous_status(is_maintenance)?;
        }

        sleep(Duration::from_secs(300)).await;
    }
}

fn is_maintenance(document: &Document) -> bool {
    for node in document.find(Class(
        "ags-ServerStatus-content-serverStatuses-server-item-label",
    )) {
        if let Some(aria_label) = node.attr("aria-label") {
            if aria_label.contains("Alexia server status is Maintenance") {
                return true;
            }
        }
    }
    false
}

async fn send_alert(is_maintenance: bool) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let (emoji, status_text, color) = if is_maintenance {
        (":red_circle:", "in maintenance", 14495300)
    } else {
        (":green_circle:", "live", 7909721)
    };

    let formatted_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let embed = json!({
        "embeds": [{
            "title": "Server Status Update",
            "description": format!("{} Alexia server is now {}!", emoji, status_text),
            "color": color,
            "footer": {
                "text": "Server Status Alert"
            },
            "timestamp": formatted_time
        }],
        "username": "Throne Of Liberty (Alexia) Server Status",
        "avatar_url": "https://dl1f6y24yx1ap.cloudfront.net/statics/2024-09-30/images/apple-touch-icon.png"
    });

	let webhook_url = env::var("TLSSA_WEBHOOK_URL").expect("webhook url is not provided");
    client.post(webhook_url).json(&embed).send().await?;
    Ok(())
}

/*
previous_status.txt
1 => online
0 => maintenance
*/
fn read_previous_status() -> Result<bool, Box<dyn Error>> {
    fs::read_to_string(".status")
        .map(|status| status.trim() == "0")
        .or_else(|_| Ok(false))
}

fn update_previous_status(is_maintenance: bool) -> Result<(), Box<dyn Error>> {
    fs::write(".status", if is_maintenance { "0" } else { "1" })?;
    Ok(())
}
