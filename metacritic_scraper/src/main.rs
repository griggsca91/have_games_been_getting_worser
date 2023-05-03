use scraper::{Html, Selector};
use chrono::{Datelike, NaiveDate};
use std::path::Path;
use std::fs::File;
use pbr::ProgressBar;


#[derive(Debug, serde::Serialize)]
struct Game {
    title: String,
    platform: String,
    meta_score: f32,
    user_score: f32,
    date_published: NaiveDate,
    year: i32,
}

fn parse_document(body: String) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(&body);
    let table_selector = Selector::parse(".clamp-summary-wrap")?;
    let title_selector = Selector::parse("a.title > h3")?;
    let platform_selector = Selector::parse(".platform > .data")?;
    let date_selector = Selector::parse(".clamp-details > span")?;
    let meta_score_selector = Selector::parse(".clamp-metascore > .metascore_anchor > .metascore_w")?;
    let user_score_selector = Selector::parse(".clamp-userscore > .metascore_anchor > .metascore_w")?;

    let mut games = vec![];

    for game_row in document.select(&table_selector) {
        let title_element = game_row.select(&title_selector).next();
        let platform_element = game_row.select(&platform_selector).next();
        let date_element = game_row.select(&date_selector).next();
        let meta_score_element = game_row.select(&meta_score_selector).next();
        let user_score_element = game_row.select(&user_score_selector).next();

        // println!("title: {:?}", title_element.unwrap().inner_html());
        // println!("meta_score: {:?}", meta_score_element.unwrap().inner_html());
        // println!("user_score: {:?}", user_score_element.unwrap().inner_html());
        let title = title_element.unwrap().inner_html().trim().to_string();
        let platform = platform_element.unwrap().inner_html().trim().to_string();
        // November 23, 1998
        let date_published = NaiveDate::parse_from_str(date_element.unwrap().inner_html().as_str(), "%B %e, %Y")?;
        let meta_score: f32 = meta_score_element.unwrap().inner_html().parse().unwrap();
        let user_score: f32 = user_score_element.unwrap().inner_html().parse().unwrap_or(-1.0);

        let game = Game {
            title,
            platform,
            meta_score,
            user_score,
            date_published,
            year: date_published.year(),
        };

        games.push(game);
    }

    Ok(games)

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("output.csv");
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    let user_agent = r#""Chromium";v="112", "Google Chrome";v="112", "Not:A-Brand";v="99""#;
    let mut wtr = csv::Writer::from_writer(file);


    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()?;

    let mut pb = ProgressBar::new(203);
    pb.format("╢▌▌░╟");

    let mut all_games: Vec<Game> = vec![];
    for page in 0..203 {
        let url = format!("https://www.metacritic.com/browse/games/score/metascore/all/all/filtered?sort=desc&page={}", page);

        let res = client.get(url)
            .send()
            .await?;
        let body = res.text().await?;
        //println!("Body = {}", body);

        let mut games = parse_document(body)?;
        all_games.append(&mut games);
        pb.inc();
    }
    pb.finish_print("done");

    for game in all_games {
        wtr.serialize(game)?;
    }

    wtr.flush()?;


    Ok(())
}
