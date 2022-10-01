use std::sync::{Arc, Mutex};

use http_types::headers::HeaderValue;
use tide::prelude::*;
use tide::security::{CorsMiddleware, Origin};
use tide::Request;

use cacher::*;

#[derive(Debug, Deserialize)]
struct ProgramArgs {
    /// cache coherence protocol
    protocol: ProtocolKind,

    /// path to the benchmark archive, e.g. "./blackscholes_four.zip"
    input_file: String,

    /// cache size in bytes
    cache_size: usize,

    /// cache associativity
    associativity: usize,

    /// cache block size in bytes
    block_size: usize,
}

#[derive(Clone)]
struct State {
    client: Arc<Mutex<System>>,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let state = State {
        client: Arc::new(Mutex::new(System::default())),
    };

    let mut app = tide::with_state(state);
    app.with(cors);
    app.at("/load").post(load);
    app.at("/cores").get(cores);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn cores(req: Request<State>) -> Result<serde_json::value::Value, http_types::Error> {
    let state = req.state();
    let system = state.client.lock().unwrap();
    let len = system.cores.len();

    Ok(json!({
        "cores": len,
    }))
}

/// Example
/// curl localhost:8080/load -d '{ "protocol": "Mesi", "input_file": "data/01", "cache_size": 256, "associativity": 2, "block_size": 8 }'
async fn load(mut req: Request<State>) -> tide::Result {
    let ProgramArgs {
        protocol,
        input_file,
        cache_size,
        associativity,
        block_size,
    } = req.body_json().await?;

    let record_streams = match FileLoader::open(&input_file) {
        Ok(streams) => streams,
        Err(e) => {
            println!(
                "Error during loading of the supplied input file: {:?}",
                e.to_string()
            );
            std::process::exit(e.raw_os_error().unwrap_or(1));
        }
    };

    let state = req.state();
    let mut system = state.client.lock().unwrap();
    system.load(
        &protocol,
        cache_size,
        associativity,
        block_size,
        record_streams,
    );

    Ok(format!("Successfully loaded system",).into())
}
