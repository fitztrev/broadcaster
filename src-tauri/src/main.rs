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
                    Some(job) => handle_pgn(&job.api_token, &job.url, &job.file),
                    None => std::thread::sleep(std::time::Duration::from_secs(1)),
                }

                app_handle
                    .emit_all("upload-queue-updated", ())
                    .expect("failed to emit event");
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

fn handle_pgn(api_token: &str, url: &str, file: &str) {
    let mut file = match File::open(file) {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening file: {}", err);
            return;
        }
    };

    let mut file_content = String::new();
    if let Err(err) = file.read_to_string(&mut file_content) {
        println!("Error reading file: {}", err);
        return;
    }

    match post_pgn_to_lichess(api_token, url, file_content) {
        Ok(_) => println!("PGN file successfully pushed to Lichess!"),
        Err(err) => eprintln!("Error pushing PGN file to Lichess: {}", err),
    }
}

fn post_pgn_to_lichess(
    api_token: &str,
    url: &str,
    pgn_content: String,
) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let request_builder: RequestBuilder = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_token))
        .body(pgn_content);

    let response = request_builder.send()?;

    // print the response body
    println!("{:#?}", response.text()?);

    Ok(())
}

#[tauri::command]
fn open_path(path: String) {
    open::that_in_background(path);
}

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
