use poem::{handler, http::StatusCode, web::{headers::{authorization::{Basic, Credentials}, Authorization, HeaderMap}, Data, Json, Path, TypedHeader}, IntoResponse};
use rand::{distributions::Alphanumeric, random, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, PgPool, Postgres};
use tokio::io::repeat;

use crate::open_tt::Map;
mod netutils;


#[handler]
pub async fn get_games(db_conn: Data<&PgPool>) -> String {
    let games = sqlx::query_as!(GamePubView, "SELECT game_id FROM game").fetch_all(db_conn.0).await.unwrap();

    serde_json::to_string(&games).unwrap()
}


#[derive(Debug, Serialize, Deserialize)]
struct GamePubView {
    game_id: String
}


// Handler for posting to the games list, thus creating a new game with the given join code and room size
#[handler]
pub async fn post_games(
        db_conn: Data<&PgPool>, 
        body: Json<GamePostRequest>) -> Result<GamePostResult, StatusCode> {
    println!("Posting new game");
    
    // Step 1: Create the new game ID
    let game_id :String = 
        rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

  
    // Step 2: Insert new record for game
    let q_result = sqlx::query!(
        "
        INSERT INTO game (game_id, join_code, max_players) 
        VALUES ($1, $2, $3)
        ",
        game_id.as_str(),
        body.join_code,
        i32::from(body.max_players)
    ).execute(db_conn.0).await.unwrap();

    // Step 3: Insert new player for Admin 
    let p_reg = netutils::register_player_for_game(db_conn.0, &game_id, "Admin".to_string()).await;

    let reg_result = match p_reg {
        Ok(res) => res,
        Err(e) => match e {
            netutils::PlayerCreationError::DatabaseError(de) => {return Err(StatusCode::BAD_REQUEST)},
            netutils::PlayerCreationError::GameDoesNotExist => {return Err(StatusCode::BAD_REQUEST)},
            netutils::PlayerCreationError::GameFull => {return Err(StatusCode::BAD_REQUEST)}
        }
    };

    // Step 4: Set game admin to newly created player
    let _ = sqlx::query!(
        "
        UPDATE game
        SET admin_id = $1
        WHERE game_id = $2
        ",
        &reg_result.p_id,
        &game_id
    ).execute(db_conn.0).await;
    
    // Step 5: Build return and set it off
    let player_passcode = reg_result.p_pass;
    let player_id = reg_result.p_id;
    return Ok(GamePostResult {game_id, admin_player: PlayerPostResponce{player_id, player_passcode} });
}

#[derive(Debug, Serialize, Deserialize)]
struct GamePostRequest {
    join_code: Option<String>,
    max_players: u8
}

#[derive(Debug, Serialize)]
struct GamePostResult {
    game_id : String,
    admin_player : PlayerPostResponce
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
    TypedHeader(p_auth) : TypedHeader<Authorization<Basic>>
) -> StatusCode {
    // Step 1: Check if sender is authorized as admin of given game
    if !netutils::is_authorized_admin(db_conn.0, &game_id, &p_auth.0).await {
        return StatusCode::FORBIDDEN;
    }

    
    // Step 2: Create a querry builder with proper head
    let mut q_builder: sqlx::query_builder::QueryBuilder<Postgres> = sqlx::query_builder::QueryBuilder::new(
        "UPDATE game SET "
    );

    // Step 3: Create a seperated builder to push all the comma seperated updates
    let mut seperated = q_builder.separated(",");

    if let Some(new_map) = body.0.new_layout {
        seperated.push("game_layout = ");
        seperated.push_bind_unseparated(serde_json::to_string(&new_map).unwrap());
        
    }

    // Step 4: Finish up query and execute 
    q_builder.push(" WHERE game_id = ");
    q_builder.push_bind(game_id);

    q_builder.build().execute(db_conn.0).await;

    return StatusCode::OK;
}

#[derive(Debug, Serialize, Deserialize)]
struct GamePatchRequest {
    new_layout: Option<Map>
}


// Handler for posting a player
#[handler]
pub async fn post_player(
    db_conn: Data<&PgPool>,
    Path(game_id): Path<String>,
    r_body: Json<PlayerPostRequest>
) -> String {
    struct GetPasscodeResult{join_code: Option<String>};

    // Step 1: Verify that the player has sent the correct passcode if there is one
    let passcode = match sqlx::query_as!(GetPasscodeResult,
    "
        SELECT join_code 
        FROM game
        WHERE game_id = $1
    ",
        &game_id
    ).fetch_one(db_conn.0).await {
        Ok(p) => p,
        Err(_) => {return "Error Crating Player, Game does not exist".to_string();}
    };

    let passcode_needed = passcode.join_code.is_some();

    if passcode_needed && passcode.join_code.unwrap_or("".to_string()) != r_body.0.join_code.unwrap_or("".to_string()) {
        return "Error Crating Player, Invalid Join Code".to_string();
    }


    // Step 2: Try to register the player
    let p_reg = netutils::register_player_for_game(
        db_conn.0, 
        &game_id, 
        r_body.0.player_name).await;

    let reg_result = match p_reg {
        Ok(res) => res,
        Err(e) => match e {
            netutils::PlayerCreationError::DatabaseError(de) => {return de.message().to_string();},
            netutils::PlayerCreationError::GameDoesNotExist => {return "Error Crating Player, Game does not exist".to_string();},
            netutils::PlayerCreationError::GameFull => {return "Error Creating Player, Game Full".to_string();}
        }
    };
    
    let player_passcode = reg_result.p_pass;
    let player_id = reg_result.p_id;

    return serde_json::to_string(&PlayerPostResponce{player_id, player_passcode}).unwrap();
}

#[derive(Debug, Deserialize)]
struct PlayerPostRequest {
    join_code: Option<String>,
    player_name: String
}

#[derive(Debug, Serialize)]
struct PlayerPostResponce {
    player_id: i32,
    player_passcode: String
}


#[handler]
pub async fn get_game(
    db_conn: Data<&PgPool>, 
    Path(game_id): Path<String>, 
    TypedHeader(auth_header) : TypedHeader<Authorization<Basic>>
) -> String {
    // Check if user is in the given game and if they are authorized
    let is_user = 
        netutils::is_player_in_game(db_conn.0, &game_id, &auth_header.0.username().to_string()).await 
        && netutils::check_user_auth(db_conn.0, auth_header.0).await.is_ok();
    serde_json::to_string(&netutils::get_game_data(&db_conn.0, &game_id, is_user).await).unwrap()
}


#[handler]
pub async fn delete_game(
    db_conn: Data<&PgPool>, 
    Path(game_id): Path<String>, 
    TypedHeader(p_auth) : TypedHeader<Authorization<Basic>>
) -> StatusCode {
    // Step 1: Check if user is Admin
    if !netutils::is_authorized_admin(db_conn.0, &game_id, &p_auth.0).await {
        return StatusCode::FORBIDDEN
    }

    // Step 2: Drop all players that are in this game
    let r1 = sqlx::query!(
        "
        DELETE 
        FROM player 
        WHERE game = $1
        ", &game_id
    ).execute(db_conn.0).await;

    // Step 2.1: Send back error if there was a problem droping the players
    match r1 {
        Err(e) => { 
            return StatusCode::INTERNAL_SERVER_ERROR;
        },
        _=>{}
    }

    // Step 3: Drop the game it'self
    let r2 = sqlx::query!(
        "
        DELETE
        FROM game
        WHERE game_id = $1
        ", &game_id
    );

    // Step 3.1: Send back error if there was a problem droping the players
    match r1 {
        Err(e) => { 
            return StatusCode::INTERNAL_SERVER_ERROR;
        },
        _=>{}
    }

    return StatusCode::OK;
}