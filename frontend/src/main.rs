use async_std::sync::Mutex;
use http_types::headers::HeaderValue;
use logger::*;
use tide::prelude::*;
use tide::security::{CorsMiddleware, Origin};
use tide::Request;

type JsonResponse = Result<serde_json::value::Value, http_types::Error>;

#[macro_use]
extern crate lazy_static;

use ::logger::Logger;

lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::open_read("../binlog"));
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
    let mut logger = LOGGER.lock().await;
    *logger = Logger::open_read("../binlog");
    let next_type = logger.read_next_type().unwrap();
    assert!(next_type == EntryType::EnvInfo);
    Ok(json!((next_type, logger.read_next::<EnvInfo>().unwrap())))
}

async fn next(_: Request<()>) -> JsonResponse {
    let mut logger = LOGGER.lock().await;
    let next_type = logger.read_next_type().unwrap();
    return match next_type {
        EntryType::EnvInfo => Ok(json!((
            EntryType::EnvInfo,
            logger.read_next::<EnvInfo>().unwrap()
        ))),
        EntryType::CoreInit => Ok(json!((
            EntryType::CoreInit,
            logger.read_next::<CoreInit>().unwrap()
        ))),
        EntryType::CacheAccess => Ok(json!((
            EntryType::CacheAccess,
            logger.read_next::<CacheAccess>().unwrap()
        ))),
        EntryType::CacheState => Ok(json!((
            EntryType::CacheState,
            logger.read_next::<CacheState>().unwrap()
        ))),
        EntryType::Step => Ok(json!((
            EntryType::Step,
            logger.read_next::<Step>().unwrap()
        ))),
        EntryType::CacheUpdate => Ok(json!((
            EntryType::CacheUpdate,
            logger.read_next::<CacheUpdate>().unwrap()
        ))),
        EntryType::CoreState => Ok(json!((
            EntryType::CoreState,
            logger.read_next::<CoreState>().unwrap()
        ))),
    };
}
