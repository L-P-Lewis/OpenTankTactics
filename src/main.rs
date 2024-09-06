pub mod open_tt;
pub mod netcode;

use poem::{get, listener::TcpListener, patch, post, EndpointExt, Route, Server};
use sqlx::PgPool;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cpool: PgPool = match PgPool::connect("postgres://postgres:password@localhost/OTTTest").await {
        Ok(p) => {
            println!("Connected to database");
            p
        },
        Err(e) => {
            println!("Error '{:?}' while connecting to database", e);
            panic!();
        }
    };
    let _ =  sqlx::migrate!("./migrations").run(&cpool).await;


    let app = Route::new()
        .at("games", 
            post(netcode::post_games)
            .get(netcode::get_games))
        .at("/games/:game_id", 
            patch(netcode::patch_game)
            .get(netcode::get_game)
            .delete(netcode::delete_game))
        .at("/games/:game_id/players", 
            post(netcode::post_player))
        .data(cpool);

    let _ = Server::new(TcpListener::bind("127.0.0.1:7878"))
        .run(app)
        .await;

    return Ok(());
}