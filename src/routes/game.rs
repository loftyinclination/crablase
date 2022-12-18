use crate::entities::chronicler;
use crate::entities::chronicler::BlaseballGameUpdate;
use crate::routes::routes;
use askama::Template;
use chrono::{DateTime, Utc};
use rocket::get;
use rocket::http::ContentType;
use rocket::response::{content::RawHtml, Debug};
use std::collections::BTreeMap;
use uuid::Uuid;

pub type ResponseResult<T> = std::result::Result<T, Debug<anyhow::Error>>;

const DEFAULT_NUMBER_OF_BASES: u8 = 3;

#[get("/game.css")]
pub fn game_css() -> (ContentType, &'static str) {
    (ContentType::CSS, routes::asset!("/game.css"))
}

#[get("/game/<game_id>")]
pub async fn game(game_id: Uuid) -> ResponseResult<Option<RawHtml<String>>> {
    let game_data = chronicler::get_game_updates(game_id).await?;

    log::info!(
        "got game data for game {}, {} updates",
        game_id,
        game_data.len()
    );

    let mut is_first = true;
    let mut away_team_id: Option<Uuid> = None;
    let mut home_team_id: Option<Uuid> = None;
    let mut weather: Option<Weather> = None;

    let mut innings: BTreeMap<(i16, bool), Inning> = BTreeMap::new();

    let mut last_inning = -1;
    let mut was_last_update_at_top_of_inning = true;
    let mut hash_for_last_update: Option<Uuid> = None;

    // FIXME: is this robust against cancelled games?
    for update in game_data {
        if is_first {
            // FIXME: support team incins
            away_team_id = Some(update.data.away_team_id);
            home_team_id = Some(update.data.home_team_id);
        }

        log::info!(
            "considering update hash={}, inning={}, is_top={}, last_update={}",
            update.hash,
            update.data.inning,
            update.data.is_top,
            update.data.last_update
        );

        is_first = false;
        last_inning = update.data.inning;

        if hash_for_last_update.is_some() && hash_for_last_update.unwrap() == update.hash {
            log::info!("discarding update");
            continue;
        }

        hash_for_last_update = Some(update.hash);

        weather = weather
            .filter(|current| current.index == update.data.weather)
            .or(Some(get_weather_for_index(update.data.weather)));

        let current_inning = innings
            // update.data.is_top is negated because true > false
            .entry((update.data.inning, !update.data.is_top))
            .or_insert_with(|| {
                log::info!(
                    "creating new inning, inning={}, is_top={}",
                    update.data.inning,
                    update.data.is_top
                );
                let pitcher_name = (if update.data.is_top {
                    update.data.home_pitcher_id.map(get_player_name)
                } else {
                    update.data.away_pitcher_id.map(get_player_name)
                })
                .unwrap_or(Some("UNKNOWN_PLAYER".to_string()));

                Inning {
                    index: update.data.inning,
                    is_top: update.data.is_top,
                    updates: Vec::new(),
                    pitcher_name,
                }
            });

        let batter = (if update.data.is_top {
            update
                .data
                .home_batter_id
                .map(|id| get_player_name(id).map(|name| Player { name }))
        } else {
            update
                .data
                .away_batter_id
                .map(|id| get_player_name(id).map(|name| Player { name }))
        })
        .flatten();

        let bases = pack_base(&update.data);

        let displayable_update = Update {
            gamelog: update.data.last_update.clone(),
            timespan: "00:00".to_string(),
            away_team_score: update.data.away_score.unwrap_or_default(),
            home_team_score: update.data.home_score.unwrap_or_default(),
            important: false,
            batter,
            bases,
        };

        current_inning.updates.push(displayable_update);
    }

    let away_team = get_team(away_team_id.expect("no away team"));
    let home_team = get_team(home_team_id.expect("no home team"));

    let page = GamePage {
        season_one_indexed: 23,
        season_zero_indexed: 22,
        day_one_indexed: 108,
        away_team,
        home_team,
        weather: weather.unwrap(),
        start_time: DateTime::parse_from_rfc3339("2021-07-24T15:00:08.61044Z")
            .map_err(anyhow::Error::from)?
            .with_timezone(&Utc),
        innings: innings.values().cloned().collect(),
    };

    Ok(Some(RawHtml(page.render().map_err(anyhow::Error::from)?)))
}

// FIXME: pull from chronicler
fn get_team(team_id: Uuid) -> Team {
    Team {
        name: "Snunbeams".to_string(),
        src: "/sun".to_string(),
        emoji: "🌞".to_string(),
    }
}

fn get_player_name(player_id: Uuid) -> Option<String> {
    Some(player_id.to_string())
}

fn get_weather_for_index(index: u8) -> Weather {
    Weather {
        index,
        name: "Flooding".to_string(),
        src: "/flooding".to_string(),
        emoji: "🚰".to_string(),
    }
}

fn pack_base(update: &BlaseballGameUpdate) -> Vec<Base> {
    let number_of_bases_they_should_have = (if update.is_top {
        update.home_bases
    } else {
        update.away_bases
    }).unwrap_or(DEFAULT_NUMBER_OF_BASES);
    let highest_occupied_base = update.bases_occupied.iter().max().map_or(0, |n| *n);

    let actual_number_of_bases = if number_of_bases_they_should_have < highest_occupied_base
    {
        highest_occupied_base
    } else {
        number_of_bases_they_should_have
    };

    log::debug!("number_of_bases={}, number_of_bases_they_should_have={}, highest_occupied_base={}",
        actual_number_of_bases,
        number_of_bases_they_should_have,
        highest_occupied_base,
    );

    let mut bases: Vec<Base> = Vec::with_capacity(actual_number_of_bases.into());
    bases.resize_with(actual_number_of_bases.into(), || Base { runners: Vec::new() } );

    for (base_index, baserunner_id) in update.bases_occupied.iter().zip(update.base_runners.clone()) {
        let base_index: usize = (*base_index).into();
        let baserunner_name = get_player_name(baserunner_id).unwrap_or("UNKNOWN PLAYER".to_string());
        bases[base_index].runners.push(baserunner_name);
    }

    bases
}

#[derive(Template)]
#[template(path = "pages/game_page.html", escape = "none")]
struct GamePage {
    season_one_indexed: i8,
    season_zero_indexed: i8,
    day_one_indexed: u8,
    away_team: Team,
    home_team: Team,
    weather: Weather,
    start_time: DateTime<Utc>,
    innings: Vec<Inning>,
}

struct Team {
    name: String,
    src: String,
    emoji: String,
}

#[derive(Clone)]
pub struct Player {
    name: String,
}

struct Weather {
    index: u8,
    name: String,
    src: String,
    emoji: String,
}

#[derive(Clone)]
struct Inning {
    index: i16,
    is_top: bool,
    updates: Vec<Update>,
    pitcher_name: Option<String>,
}

#[derive(Clone)]
pub struct Update {
    gamelog: String,
    timespan: String,
    away_team_score: f32,
    home_team_score: f32,
    important: bool,
    batter: Option<Player>,
    bases: Vec<Base>,
}

#[derive(Clone)]
pub struct Base {
    runners: Vec<String>,
}
