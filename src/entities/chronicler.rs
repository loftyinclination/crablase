use reqwest::Client;
use serde::Deserialize;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref CLIENT: Client = Client::builder()
        .user_agent("reblase/2.0 (lofty@sibr.dev")
        .build()
        .unwrap();
}

const CHRONICLER_BASE: &str = "https://api.sibr.dev/chronicler/";

pub async fn get_game_updates(game_id: Uuid) -> Result<Vec<ChronGameUpdate>, anyhow::Error> {
    // FIXME: work for semicentennial
    let url = format!("{}v1/games/updates?game={}&count=1000", CHRONICLER_BASE, game_id.to_string());

    log::info!("requesting data for game with id {} from {}", game_id, url);

    let requested_game_updates: Chron1Response<ChronGameUpdate> = CLIENT.get(url).send().await?.json().await?;

    log::info!("got data for game {}", game_id);

    // FIXME: should perform more requests if next_page is not none
    Ok(requested_game_updates.data)
}

pub async fn get_team_version(team_id: Uuid) -> Result<ChronTeam, anyhow::Error> {
    let url = format!("{}/v2/entities?type=Team&id={}", CHRONICLER_BASE, team_id.to_string());

    log::info!("requesting entity for team with id {}", team_id);

    let team_data: ChronTeam = CLIENT.get(url).send().await?.json::<Chron2Response<ChronTeam>>().await?.items.remove(1);

    log_got_data(&team_data);

    Ok(team_data)
}

fn log_got_data(team_data: &ChronTeam) -> () {
    log::info!("got data for team {}", team_data.name_if_scattered().unwrap_or(&team_data.full_name));
}

#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"), default)]
struct Chron1Response<T> {
    next_page: Option<String>,
    data: Vec<T>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"), default)]
pub struct ChronGameUpdate {
    pub game_id: Uuid,
    pub hash: Uuid,
    pub timestamp: String,
    pub data: BlaseballGameUpdate,
}

#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"), default)]
pub struct BlaseballGameUpdate {
    #[serde(alias = "_id")]
    pub id: Uuid,

    pub outcomes: Option<Vec<String>>,
    pub last_update: String,

    pub sim: Option<String>,
    pub season: u8,
    pub day: u8,
    pub is_postseason: bool,
    pub weather: u8,
    pub stadium: Option<Uuid>,

    // only ever observed to be a i8... but what if casino zone
    pub inning: i16,
    #[serde(rename = "topOfInning")]
    pub is_top: bool,
    pub play_count: u16,
    pub shame: bool,

    pub away_score: Option<f32>,
    pub home_score: Option<f32>,

    pub half_inning_outs: u8,
    pub at_bat_balls: u8,
    pub at_bat_strikes: u8,
    pub baserunner_count: u8,
    pub bases_occupied: Vec<u8>,
    pub base_runners: Vec<Uuid>,

    pub away_team_id: Uuid,
    pub away_pitcher_id: Option<Uuid>,
    pub away_batter_id: Option<Uuid>,

    pub away_bases: Option<u8>,
    pub away_balls: Option<u8>,
    pub away_strikes: Option<u8>,
    pub away_outs: Option<u8>,

    pub home_team_id: Uuid,
    pub home_pitcher_id: Option<Uuid>,
    pub home_batter_id: Option<Uuid>,

    pub home_bases: Option<u8>,
    pub home_balls: Option<u8>,
    pub home_strikes: Option<u8>,
    pub home_outs: Option<u8>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"), default)]
struct Chron2Response<T> {
    next_page: Option<String>,
    items: Vec<T>,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"), default)]
struct ChronEntityVersion<T> {
    entity_id: Uuid,
    hash: Uuid,
    valid_from: String,
    valid_to: Option<String>,
    items: Vec<T>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"), default)]
struct ChronTeam {
    full_name: String,
    emoji: String,

    stadium: Option<Uuid>,

    state: Option<ChronTeamState>,
}

impl ChronTeam {
    fn name_if_scattered(&self) -> Option<&String> {
        Some(&self.state.as_ref()?.scattered.as_ref()?.full_name)
    }
}

#[derive(Deserialize)]
struct ChronTeamState {
    scattered: Option<TeamScatteredState>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"), default)]
struct TeamScatteredState {
    full_name: String,
}
