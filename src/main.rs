pub mod open_tt;

use open_tt::{BoardPos, PlayerTank};
use poem::{get, handler, listener::TcpListener, web::Path, IntoResponse, Route, Server};


// #[handler]
// fn hello(Path(name): Path<String>) -> String {
//     format!("hello: {}", name)
// }

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // let app = Route::new().at("/hello/:name", get(hello));
    // Server::new(TcpListener::bind("127.0.0.1:7878"))
    //     .run(app)
    //     .await
    let p_test = PlayerTank {
        position: BoardPos(2, 5),
        ..Default::default()
    };

    let p_ser = serde_json::to_string(&p_test).expect("msg");

    let p_recon : PlayerTank = serde_json::from_str(&p_ser).expect("msg");

    assert_eq!(p_recon, p_test);

    println!("{}", p_ser);

    return Ok(());
}