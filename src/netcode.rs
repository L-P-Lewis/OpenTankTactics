use poem::{handler, http::StatusCode, web::{headers::HeaderMap, Data, Json, Path}, IntoResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::open_tt::Map;


// Handler for posting to the games list, thus creating a new game with the given join code and room size
#[handler]
pub async fn post_games(
        db_conn: Data<&PgPool>, 
        body: Json<GamePostRequest>) -> Result<GamePostResult, poem::error::GetDataError> {
    todo!()
}

#[derive(Debug, Serialize, Deserialize)]
struct GamePostRequest {
    join_code: Option<String>,
    max_players: u8
}

#[derive(Debug, Serialize, Deserialize)]
struct GamePostResult {
    game_id : String,
    admin_code : String
}

impl IntoResponse for GamePostResult {
    fn into_response(self) -> poem::Response {
        serde_json::to_string(&self).unwrap().into_response()
    }
}

// Handler for patching the data of a game, patch must be done by the admin
#[handler]
pub async fn patch_game(
    db_conn: Data<&PgPool>,
    Path(game_id): Path<String>,
    body: Json<GamePatchRequest>,
    header : &HeaderMap
) -> StatusCode {
    todo!()
}

#[derive(Debug, Serialize, Deserialize)]
struct GamePatchRequest {
    new_layout: Option<Map>
}