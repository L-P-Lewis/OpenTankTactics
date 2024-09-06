use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use sqlx::{query, query_as, Error, PgPool};
use poem::web::headers::authorization;

pub async fn check_user_auth(db_conn: &PgPool, auth_data: authorization::Basic) -> Result<(), UserAuthError> {
    let player_id = match auth_data.username().parse::<i32>() {
        Ok(p) => p,
        Err(_) => {return Err(UserAuthError::PlayerIDInvalid)}
    };

    let player_pass = match sqlx::query!(
        "
        SELECT passcode
        FROM player
        WHERE player_id = $1
        ", &player_id
    ).fetch_one(db_conn).await {
        Ok(r) => r.passcode,
        Err(_) => {return Err(UserAuthError::PlayerIDInvalid)}
    };

    if player_pass != auth_data.password().to_string() {
        return Err(UserAuthError::PlayerPasswordInvalid);
    }
    
    return Ok(())
}

pub enum UserAuthError {
    PlayerIDInvalid,
    PlayerPasswordInvalid,
}


// Returns metadata about a game lobby
pub async fn get_game_data(db_conn: &PgPool, game_id: &String, as_user: bool) -> GameData {
    let players = match as_user {
        true => Some(get_players_in_game(db_conn, game_id).await.unwrap_or(Vec::new())),
        false => None
    };

    let space = get_game_capacity(db_conn, game_id).await.unwrap_or((0, 0));

    GameData{game_id: game_id.to_string(), space, players}
}

#[derive(Debug, Serialize)]
pub struct GameData {
    game_id: String,
    space: (u8, u8),
    players: Option<Vec<String>>
}

// Gets the capacity of a game and the number of active players
// Returns (max_players, current_players)
pub async fn get_game_capacity(db_conn: &PgPool, game_id: &String) -> Result<(u8, u8), Box<dyn sqlx::error::DatabaseError>> {
    let mp_query = sqlx::query!( "SELECT max_players FROM game WHERE game_id = $1", game_id).fetch_one(db_conn).await;

    let max_player_count: u8 = match mp_query {
        Ok(r) => r.max_players.unwrap_or(0).try_into().unwrap(),
        Err(e) => {return Err(e.into_database_error().expect("query!() managed to return a non-database error, which is a big problem.")); }
    };

    let cp_query = sqlx::query!(
        "
        SELECT count(*)
        FROM player
        WHERE game = $1
        "
        ,game_id
    ).fetch_one(db_conn).await;
    let current_player_count: u8 = match cp_query {
        Ok(r) => r.count.unwrap_or(0).try_into().unwrap(),
        Err(e) => {return Err(e.into_database_error().expect("query!() managed to return a non-database error, which is a big problem.")); }
    };

    return Ok((max_player_count, current_player_count));
}


// Returns a list of players associated with a given game
pub async fn get_players_in_game(db_conn: &PgPool, game_id: &String) -> Result<Vec<String>, Error> {
    match sqlx::query!(
        "
        SELECT player_id
        FROM player
        WHERE game = $1
        ", game_id
    ).fetch_all(db_conn).await {
        Err(e) => Err(e),
        Ok(list) => Ok(list.iter().map(|rec| rec.player_id.to_string()).collect())
    }
}


// Adds a new player to the database and points it at the given game
pub async fn register_player_for_game(db_conn: &PgPool, game_id: &String, name: String) -> Result<NewPlayer, PlayerCreationError> {
    // Step 1: Check if game is full
    let capacity :u8= match get_game_capacity(db_conn, &game_id).await {
        Ok((m, c)) => m - c,
        Err(e) => {return Err(PlayerCreationError::DatabaseError(e));}
    };

    if capacity == 0 {
        return Err(PlayerCreationError::GameFull);
    }

    // Step 2: If not full, create a new player entry pointed at the game
    let p_pass :String = 
        rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    let create_player_result = sqlx::query!(
        "
        INSERT INTO player (player_name, passcode, game)
        VALUES ($1, $2, $3)
        RETURNING player_id
        ",
        &name,
        &p_pass,
        &game_id
    ).fetch_one(db_conn).await;

    let p_id: i32 = match create_player_result {
        Ok(r) => r.player_id,
        Err(e) => {return Err(PlayerCreationError::DatabaseError(e.into_database_error().expect("query!() managed to return a non-database error, which is a big problem.")));}
    };


    // Setp 3: Compile data for output
    return Ok(NewPlayer { p_id, p_pass})
}

pub struct NewPlayer {pub p_id : i32, pub p_pass : String}

pub enum PlayerCreationError {
    GameDoesNotExist,
    GameFull,
    DatabaseError(Box<dyn sqlx::error::DatabaseError>)
}


pub async fn is_player_in_game(db_conn: &PgPool, game_id: &String, player_id: &String) -> bool {
    (match sqlx::query!(
        "
        SELECT game
        FROM player
        WHERE player_id = $1
        ",
        player_id.parse::<i32>().unwrap_or(-1)
    ).fetch_one(db_conn).await {
        Ok(rec) => rec.game,
        Err(_) => "".to_string()
    }) == game_id.to_string()

}


// Combined function to check both user auth and wether or not a player belongs to the given game
// Since these are going to be done a lot together it seems wise to combine these into a single api call
pub async fn is_authorized_player(db_conn: &PgPool, game_id: &String, p_auth : &authorization::Basic) -> bool {
    // Step 1: Extract player ID from auth data
    let player_id = match p_auth.username().parse::<i32>() {
        Ok(p) => p,
        Err(_) => {return false}
    };

    // Step 2: Querry player data from database
    struct PQuerry {
        passcode: String,
        game: String
    }

    let p_data = match sqlx::query_as!(PQuerry,
        "
        SELECT passcode, game
        FROM player
        WHERE player_id = $1
        ", &player_id
    ).fetch_one(db_conn).await {
        Ok(r) => r,
        Err(_) => {return false}
    };

    // Check passcode and game 
    if p_data.passcode != p_auth.password().to_string() {
        return false;
    }

    if p_data.game != game_id.to_string() {
        return false;
    } 
    
    return true;
}


// Combined method for checking if given auth corresponds to the admin of a game
// Similar reasoning to above
pub async fn is_authorized_admin(db_conn: &PgPool, game_id: &String, p_auth : &authorization::Basic) -> bool {
    // Step 1: Check if player is authorized as part of given game
    if !is_authorized_player(db_conn, game_id, p_auth).await {
        return false;
    }

    // Step 2: Get database admin info
    struct DQuery {
        admin_id: Option<i32> // Quick hack to make sqlx error checking not scream at me since I didn't declare admin ID as NOT NULL and I'm not refreshing the database right now, remove later
    }
    let q_result = sqlx::query_as!(DQuery,
        "
        SELECT admin_id
        FROM game
        WHERE game_id = $1
        ", game_id
    ).fetch_one(db_conn).await;
    if let Err(_) = q_result {
        return false;
    }
    let a_id = q_result.unwrap().admin_id.unwrap_or(-1);

    // Step 3: Check if player is admin
    if a_id != p_auth.username().parse::<i32>().unwrap_or(-1) {
        return false;
    }

    return true;
}