#![deny(clippy::unwrap_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

use chrono::Local;
use poem::{
    get, handler, http::HeaderMap, listener::TcpListener, web::{Path, RealIp}, Request, Route, Server
};
use tracing::{debug, info, instrument, Level};
use tracing_subscriber::EnvFilter;

mod cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let filter = EnvFilter::builder()
        .parse("phisher=info,poem=info,tokio=warn")
        .expect("Could not create env filter.");

    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::TRACE)
        .with_target(true)
        .with_env_filter(filter)
        .with_thread_ids(false)
        .with_file(false)
        .without_time()
        .init();

    cli::parse().await
}

#[instrument(level="trace", name="web" skip_all)]
async fn run_server() -> Result<(), std::io::Error> {

    info!("Request data is put on stdout, you can safely redirect it to a file without it getting clobbered by `./foo 2> bar`");

    let app = Route::new()
        .at("/:id", get(gotcha))
        ;
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}

#[handler]
fn gotcha(Path(path): Path<String>, ip: RealIp, req: &Request) {
    
    fn get_header(header: &str, headers: &HeaderMap) -> String {
        headers
            .get(header)
            .map_or(format!("No {header}"), |f| f
                    .to_str()
                    .ok()
                    .map_or(format!("{header} has bad data"), |n| n.to_string())
                    )
    }

    let time = Local::now(); 
    let ip = ip.0.map(|f| f.to_string()).unwrap_or("Invalid IP".to_string());
    let headers = req.headers();

    let computer_name = get_header("X-ComputerName", headers);
    let username = get_header("X-Username", headers);
    let user_agent = get_header("user-agent", headers);
    let uri = req.uri();
 
    debug!("Hit from {username}");
    eprintln!("{time}|{path}|{ip}|{uri}|{username}|{computer_name}|{user_agent}")
}
