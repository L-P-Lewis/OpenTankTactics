pub mod open_tt;
pub mod netcode;

use poem::{get, listener::TcpListener, patch, post, EndpointExt, Route, Server};
use sqlx::PgPool;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cpool: PgPool = PgPool::connect("postgres://postgres:password@localhost/OTTTest").await.unwrap();
    
    let app = Route::new()
        .at("games", post(netcode::post_games))
        .at("/games/:game_id", patch(netcode::patch_game))
        .data(cpool);

    Server::new(TcpListener::bind("127.0.0.1:7878"))
        .run(app)
        .await?;
    return Ok(());
}