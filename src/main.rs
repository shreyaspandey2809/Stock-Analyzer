use std::io::{self, Write};
use std::fs::File;
use std::process::Command;
use plotters::prelude::*;
use serde_json::{Value, json};
use chrono::NaiveDate; 
use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter stock symbol: ");
    io::stdout().flush()?;
    let mut symbol = String::new();
    io::stdin().read_line(&mut symbol)?;
    let symbol = symbol.trim().to_string();

    let url = format!(
        "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol={}&apikey=MD6KR6R3WO38A4C0",
        symbol
    );

    println!("Fetching stock data...");
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?;

    let body = res.text().await?;
    let json: Value = serde_json::from_str(&body)?;

    let mut data: Vec<(NaiveDate, f64)> = Vec::new();

    if let Some(series) = json.get("Time Series (Daily)").and_then(|s| s.as_object()) {
        data = series
            .iter()
            .filter_map(|(date_str, values)| {
                let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
                let close = values["4. close"].as_str()?.parse::<f64>().ok()?;
                Some((date, close))
            })
            .collect();
    } else {
        eprintln!("Error: Alpha Vantage response is missing 'Time Series (Daily)'.");
        return Ok(());
    }

    data.sort_by(|a, b| a.0.cmp(&b.0));

    let json_data = json!(
        data.iter().map(|(d, p)| (d.to_string(), *p)).collect::<Vec<_>>()
    );
    let file = File::create("stock_data.json")?;
    serde_json::to_writer_pretty(file, &json_data)?;
    println!("✅ Saved stock data to stock_data.json");

    println!("🤖 Running AI prediction...");
    let python_cmd = if cfg!(target_os = "windows") { "python" } else { "python3" };
    let status = Command::new(python_cmd)
        .arg("predict.py")
        .status()?;

    if !status.success() {
        eprintln!("❌ Error: Python script failed.");
        return Ok(());
    }

    let ai_output: Value = match std::fs::read_to_string("ai_output.json") {
        Ok(content) => serde_json::from_str(&content)?,
        Err(_) => {
            eprintln!("❌ Error: Could not read ai_output.json");
            return Ok(());
        }
    };

    let predicted_price = ai_output["predicted_price"].as_f64().unwrap_or(0.0);

    println!("\n AI Analysis:");
    println!("Trend           : {}", ai_output["trend"].as_str().unwrap_or("N/A"));
    println!("Predicted Price : {:.2}", predicted_price);
    println!("Recommendation  : {}", ai_output["recommendation"].as_str().unwrap_or("N/A"));

    let root = BitMapBackend::new("chart.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let (min_price, max_price) = data.iter().fold((f64::MAX, f64::MIN), |(min, max), x| {
        (min.min(x.1), max.max(x.1))
    });

    let min_date = data.first().unwrap().0;
    let max_date = data.last().unwrap().0;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Stock Prices: {}", symbol.to_uppercase()), ("sans-serif", 30))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min_date..max_date, min_price..max_price)?;

    chart.configure_mesh().x_labels(10).disable_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter().map(|(date, price)| (*date, *price)),
            &RED,
        ))?
        .label("Close Price")
        .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .draw_series(LineSeries::new(
            (0..data.len()).map(|i| (data[i].0, predicted_price)),
            &BLUE.mix(0.6),
        ))?
        .label(format!("Predicted Price: {:.2}", predicted_price))
        .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart.draw_series(std::iter::once(Text::new(
        format!("Predicted: {:.2}", predicted_price),
        (data.last().unwrap().0, predicted_price),
        ("sans-serif", 15).into_font().color(&BLUE),
    )))?;

    chart.configure_series_labels().border_style(&BLACK).draw()?;

    println!("📈 Chart saved as chart.png");

    Ok(())
}
