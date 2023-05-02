use scraper::{Html, Selector};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let user_agent = r#""Chromium";v="112", "Google Chrome";v="112", "Not:A-Brand";v="99""#;
    let url = "https://www.metacritic.com/browse/games/score/metascore/all/all/filtered?sort=desc";


    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()?;

    let res = client.get(url)
    .send()
    .await?;

    let body = res.text().await?;
    //println!("Body = {}", body);

    let document = Html::parse_document(&body);
    let table_selector = Selector::parse(".clamp-summary-wrap")?;

    for game in document.select(&table_selector) {
        println!("{:?}", game.value())
    }


    println!("Hello, world!");
    Ok(())
}
