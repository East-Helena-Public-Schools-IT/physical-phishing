#![deny(clippy::unwrap_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![feature(ascii_char)]
#![feature(addr_parse_ascii)]
#![feature(async_closure)]

use std::future::IntoFuture;

use account::simple_storage::FilesystemDB;
use clap::Parser;
use poem::{
    EndpointExt, endpoint::StaticFilesEndpoint, listener::TcpListener, Route, Server
};
use tracing::{Level, instrument};
use tracing_subscriber::EnvFilter;

use crate::account::AppState;

mod account;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    new: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    #[cfg(debug_assertions)]
    let filter = EnvFilter::builder()
            .parse("room_clocks=trace,poem=debug,tokio=warn")
            .expect("Could not create env filter.")
            ;

    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::fmt()
        .with_max_level(Level::TRACE)
        .with_target(true)
        .with_env_filter(filter)
        .with_thread_ids(false)
        .with_file(false)
        .without_time()
        .init();

    let args = Args::parse();

    match args.new {
        Some(a) => {
            let mut x = a.split_ascii_whitespace();
            let username = x.next().expect("You passed an empty string as credentials...");
            let password = x.next().expect("You need to pass a password along with that username.");
            // let acct = Account::new(&AccountLogin::new(password, username)).expect("Internal error whilst building account. Most likely during the password hashing process.");
            if let Ok(act) = account::Account::try_new(username, password) {
                println!("{}", act); 
            }
            return Ok(())
        },
        None => {/*continue*/},
    }

    // Tick thru messages
    run_server().into_future().await
}


pub static FILESYSTEM_ROOT: &str = "./www/";
#[instrument(level="trace", name="web" skip_all)]
async fn run_server() -> Result<(), std::io::Error> {
    let appstate = AppState::<FilesystemDB>::new().await;

    // Wow, poem routing gets a bit messy
    let app = Route::new()
        .nest("/", StaticFilesEndpoint::new(FILESYSTEM_ROOT)
            .index_file("index.html")
            .fallback_to_index()
        )

        .nest("/protected", Route::new()
            .nest("/", StaticFilesEndpoint::new(FILESYSTEM_ROOT.to_owned()+"protected/")
                .index_file("account.html") 
                .fallback_to_index()
            )
            .with(appstate)
        )
        .with(poem::middleware::Tracing)
    ;

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}

