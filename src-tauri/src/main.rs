#![warn(clippy::pedantic)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod oauth;

use std::{
    collections::VecDeque,
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::oauth::start_oauth_flow;

#[derive(Default)]
struct UploadQueue {
    jobs: VecDeque<UploadJob>,
}

#[derive(Debug)]
struct UploadJob {
    file: String,
    url: String,
    api_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PgnPushResponse {
    moves: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PgnPushResult {
    response: PgnPushResponse,
    file: String,
}

fn main() {
    let upload_queue_state = Arc::new(Mutex::new(UploadQueue::default()));
    let arced_upload_queue = Arc::clone(&upload_queue_state);

    tauri::Builder::default()
        .manage(upload_queue_state)
        .setup(|app| {
            let app_handle = app.handle();

            std::thread::spawn(move || loop {
                let mut queue = arced_upload_queue.lock().unwrap();
                println!("jobs: {:?}", queue.jobs);

                let next_job = queue.jobs.pop_front();
                drop(queue);

                match next_job {
                    Some(job) => match handle_pgn(&job) {
                        Ok(response) => {
                            app_handle
                                .emit_all(
                                    "upload_success",
                                    PgnPushResult {
                                        response,
                                        file: job.file,
                                    },
                                )
                                .expect("failed to emit event");
                        }
                        Err(err) => {
                            app_handle
                                .emit_all("upload_error", err)
                                .expect("failed to emit event");
                        }
                    },
                    None => std::thread::sleep(std::time::Duration::from_secs(1)),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_to_queue,
            open_path,
            start_oauth_flow
        ])
        .plugin(tauri_plugin_fs_watch::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn handle_pgn(job: &UploadJob) -> Result<PgnPushResponse, String> {
    let mut file = match File::open(&job.file) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };

    let mut file_content = String::new();
    if let Err(err) = file.read_to_string(&mut file_content) {
        return Err(err.to_string());
    }

    post_pgn_to_lichess(&job.api_token, &job.url, file_content)
}

fn post_pgn_to_lichess(
    api_token: &str,
    url: &str,
    pgn_content: String,
) -> Result<PgnPushResponse, String> {
    let client = Client::new();
    let request_builder: RequestBuilder = client
        .post(url)
        .header("Authorization", format!("Bearer {api_token}"))
        .body(pgn_content);

    let response = match request_builder.send() {
        Ok(response) => response,
        Err(err) => return Err(format!("Error sending request: {err}")),
    };

    if response.status().is_success() {
        match response.json::<PgnPushResponse>() {
            Ok(response) => Ok(response),
            Err(err) => Err(format!("Error parsing response: {err}")),
        }
    } else {
        Err(format!("Error pushing PGN file to Lichess: {response:?}"))
    }
}

#[tauri::command]
fn open_path(path: String) {
    open::that_in_background(path);
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
fn add_to_queue(
    state: tauri::State<'_, Arc<Mutex<UploadQueue>>>,
    api_token: &str,
    url: &str,
    files: Vec<String>,
) {
    println!("adding {} files to queue to {}", files.len(), url);

    let mut queue = state.lock().unwrap();

    for file in files {
        queue.jobs.push_back(UploadJob {
            api_token: api_token.to_string(),
            url: url.to_string(),
            file: file.to_string(),
        });
    }
}
