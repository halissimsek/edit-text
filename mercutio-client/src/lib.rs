#![feature(crate_in_paths)]

#[cfg(not(target_arch="wasm32"))]
extern crate bus;
#[cfg(not(target_arch="wasm32"))]
extern crate crossbeam_channel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate taken;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate take_mut;
#[macro_use]
extern crate lazy_static;
extern crate ron;
#[cfg(not(target_arch="wasm32"))]
extern crate ws;
extern crate colored;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
#[cfg(not(target_arch="wasm32"))]
extern crate dotenv;
#[cfg(not(target_arch="wasm32"))]
extern crate url;
extern crate mercutio;

/* logging */

// Macros can only be used after they are defined

use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use mercutio::*;

lazy_static! {
    static ref LOG_WASM_FILE: Arc<Mutex<File>> = {
        let path = Path::new("./log/client");
        Arc::new(Mutex::new(File::create(path).unwrap()))
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogWasm {
    Setup(String),
    Task(String, client::Task),
    SyncNew(String),

    SendClient(ClientCommand),
    SendSync(SyncServerCommand),
    Debug(String),
}

macro_rules! log_wasm {
    ( $x:expr ) => {
        {
            use std::io::prelude::*;
            use std::env::var;

            // Only if MERCUTIO_WASM_LOG=1 is set
            if var("MERCUTIO_WASM_LOG") == Ok("1".to_string()) {
                use $crate::LogWasm::*;
                let mut file_guard = LOG_WASM_FILE.lock().unwrap();
                let mut ron = ::ron::ser::to_string(&$x).unwrap();
                ron = ron.replace("\n", "\\n"); // Escape newlines
                let _ = writeln!(*file_guard, "{}", ron);
                let _ = file_guard.sync_data();
            }
        }
    };
}

/* /logging */

pub mod actions;
pub mod walkers;
#[cfg(not(target_arch="wasm32"))]
pub mod proxy;
pub mod state;
pub mod client;
#[cfg(not(target_arch="wasm32"))]
pub mod monkey;
pub mod random;

pub use self::client::*;
pub use self::state::*;
pub use self::actions::*;
pub use self::random::*;
