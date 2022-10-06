use http_types::headers::HeaderValue;
use logger::*;
use tide::prelude::*;
use tide::security::{CorsMiddleware, Origin};
use tide::Request;

type JsonResponse = Result<serde_json::value::Value, http_types::Error>;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref LOGGER: Logger = Logger::new();
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let mut app = tide::new();
    app.with(cors);
    app.at("/next").get(next);
    app.at("/load").get(load);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn load(_: Request<()>) -> JsonResponse {
    LOGGER.open_read("../binlog");
    match LOGGER.read() {
        Some(LogEntry::EnvInfo(env_info)) => Ok(json!(LogEntry::EnvInfo(env_info))),
        _ => panic!("Invalid log format"),
    }
}

async fn next(_: Request<()>) -> JsonResponse {
    let mut entry_list: Vec<LogEntry> = Vec::new();
    while entry_list.last().map_or(true, |last| match last {
        LogEntry::Step(_) => false,
        _ => true,
    }) {
        match LOGGER.read() {
            Some(entry) => entry_list.push(entry),
            None => break,
        }
    }
    return Ok(json!(entry_list));
}
