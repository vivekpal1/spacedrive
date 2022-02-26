use futures::{stream::StreamExt, Stream};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::fs;
use tokio::sync::mpsc;
use ts_rs::TS;

pub mod config;
pub mod crypto;
pub mod db;
pub mod file;
pub mod native;
// pub mod p2p;
pub mod util;
use futures::executor::block_on;

// static configuration passed in by host application
#[derive(Serialize, Deserialize, Debug)]
pub struct CoreConfig {
    pub data_dir: std::path::PathBuf,
    pub primary_db: std::path::PathBuf,
    pub file_type_thumb_dir: std::path::PathBuf,
}

// represents an event this library can emit
#[derive(Serialize, Deserialize, Debug, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum ClientEvent {
    NewFileTypeThumb { file_id: u32, icon_created: bool },
    NewJobCreated { job_id: u32, progress: u8 },
    DatabaseDisconnected { reason: Option<String> },
}

pub struct Core {
    pub config: CoreConfig,
    pub event_channel_sender: mpsc::Sender<ClientEvent>,
}

pub static CORE: OnceCell<Core> = OnceCell::new();

pub fn get_core_config() -> &'static CoreConfig {
    &CORE.get().unwrap().config
}

pub async fn core_send(event: ClientEvent) {
    println!("Core Event: {:?}", event);
    CORE.get()
        .unwrap()
        .event_channel_sender
        .send(event)
        .await
        .unwrap();
}

pub async fn core_send_stream<T: Stream<Item = ClientEvent>>(stream: T) {
    stream
        .for_each(|event| async {
            core_send(event).await;
        })
        .await;
}

pub fn configure(mut data_dir: std::path::PathBuf) -> mpsc::Receiver<ClientEvent> {
    data_dir = data_dir.join("spacedrive");

    let (event_sender, event_receiver) = mpsc::channel(100);

    let config = CoreConfig {
        data_dir: data_dir.clone(),
        primary_db: data_dir.clone().join("primary.db3"),
        file_type_thumb_dir: data_dir.clone().join("file_icons"),
    };

    let _ = CORE.set(Core {
        config,
        event_channel_sender: event_sender,
    });

    fs::create_dir_all(&get_core_config().data_dir).unwrap();
    fs::create_dir_all(&get_core_config().file_type_thumb_dir).unwrap();

    config::state::make(data_dir.to_str().unwrap());

    // create primary data base if not exists
    block_on(db::connection::create_primary_db()).expect("failed to create primary db");
    block_on(file::init::init_library()).expect("failed to init library");

    println!("Spacedrive daemon online");

    // p2p::listener::listen(None);

    // env_logger::builder()
    //     .filter_level(log::LevelFilter::Debug)
    //     .is_test(true)
    //     .init();

    event_receiver
}
// println!("Spacedrive test");
// use crate::file::watcher::watch_dir;
// use std::{thread, time};
// watch_dir("/Users/jamie/Downloads/");
// let duration = time::Duration::from_secs(500);
// let now = time::Instant::now();
// thread::sleep(duration);
// assert!(now.elapsed() >= duration);

pub fn main() {
    // hello!
    println!("Hello, world!");
}