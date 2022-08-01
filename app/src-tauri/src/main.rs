#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ashuk_core::{
    compresser::{compress_to_target_extension, CompressOptions, Result as CompressResult, Status},
    format_meta::{CompressOptionsContext, ImageFormat, ProcessStrategy},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use env_logger;

use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputResult {
    pub path: String,
    pub size: u64,
    pub writable_extensions: Vec<String>,
    pub extension: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileContext {
    pub status: Status,
    pub input: InputResult,
    pub output: Option<CompressResult>,
}

type FileList = HashMap<String, FileContext>;

pub struct FileState {
    files: Mutex<FileList>,
    options: Mutex<CompressOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EmitFileOperation {
    Create,
    Update,
    Compress,
    Clear,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmitFileRequestBody {
    files: Option<FileList>,
    operation: EmitFileOperation,
    options: Option<CompressOptions>,
}

impl FileState {
    pub fn new(files: FileList) -> Self {
        // set default option
        let option = get_compress_options_context().unwrap();
        Self {
            files: Mutex::new(files),
            options: Mutex::new(CompressOptions {
                extension: option[0].extension.clone(),
                quality: Some(option[0].default),
            }),
        }
    }

    pub fn add_file(&self, file_path: &str) -> FileContext {
        let mut files = self.files.lock().unwrap();
        // init data
        let file = FileContext {
            status: Status::Initialized,
            input: InputResult {
                path: file_path.to_string().clone(),
                size: std::fs::metadata(&file_path)
                    .expect("There is no file.")
                    .len(),
                writable_extensions: ImageFormat::get_formats()
                    .iter()
                    .filter(|x| ImageFormat::can_write(&x))
                    .map(|y| y.get_representative_ext_str())
                    .collect::<Vec<String>>(),
                extension: ImageFormat::from_path(&file_path)
                    .unwrap()
                    .get_representative_ext_str(),
            },
            output: Some(CompressResult {
                size: 0,
                path: "".to_string(),
                elapsed: 0,
                extension: ImageFormat::from_path(&file_path)
                    .unwrap()
                    .get_representative_ext_str(),
            }),
        };
        // update hashmap
        files.entry(file_path.to_string()).or_insert(file.clone());

        file
    }

    pub fn update_file(&self, path: String, file_status: FileContext) -> FileContext {
        let mut files = self.files.lock().unwrap();
        let new_file = FileContext {
            status: file_status.status,
            input: file_status.input,
            output: file_status.output,
        };
        // update hashmap
        files.entry(path).or_insert(new_file.clone());

        new_file
    }

    pub fn get_files(&self) -> FileList {
        let files = self.files.lock().unwrap();
        files.clone()
    }

    pub fn clear(&self) {
        let mut files = self.files.lock().unwrap();
        files.clear();
    }

    pub fn update_options(&self, _options: Option<CompressOptions>) {
        let mut options = self.options.lock().unwrap();
        *options = _options.unwrap();
    }

    pub fn process_compress(
        &self,
        app_handle: &tauri::AppHandle,
        emitter_name: &str,
        path: std::string::String,
        file: FileContext,
    ) {
        // compress option
        let options = self.options.lock().unwrap();
        // update state
        let updated_file = self.update_file(
            path.clone(),
            FileContext {
                status: Status::Pending,
                input: file.clone().input,
                output: None,
            },
        );
        // notify to client for start compressing
        notify_file_to_client(&app_handle, &updated_file, emitter_name);

        // compress image
        let result = compress_to_target_extension(
            &path,
            CompressOptions {
                extension: options.extension.clone(),
                quality: options.quality,
            },
        );

        if result.is_ok() {
            // update state
            let updated_file = self.update_file(
                path,
                FileContext {
                    status: Status::Success,
                    input: file.input,
                    output: Some(result.unwrap()),
                },
            );
            // notify to client for success
            notify_file_to_client(&app_handle, &updated_file, emitter_name);
        } else {
            // update state
            let updated_file = self.update_file(
                path,
                FileContext {
                    status: Status::Failed,
                    input: file.input,
                    output: None,
                },
            );
            // notify to client for failure
            notify_file_to_client(&app_handle, &updated_file, emitter_name);
        }
    }
}

fn notify_file_to_client<T: ?Sized + Serialize>(
    app_handle: &tauri::AppHandle,
    file_context: &T,
    emitter_name: &str,
) {
    let serialized = serde_json::to_string(&file_context).expect("Invalid data format");
    app_handle.emit_all(&emitter_name, serialized).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SupportedFormatMeta {
    ext: String,
    readable: bool,
    writable: bool,
}

#[tauri::command]
fn get_compress_options_context() -> Result<Vec<CompressOptionsContext>, String> {
    let options = ImageFormat::get_formats()
        .iter()
        .map(|x| x.get_compress_options_context())
        .collect::<Vec<CompressOptionsContext>>();
    Ok(options)
}

#[tauri::command]
fn get_supported_extensions() -> Result<Vec<SupportedFormatMeta>, String> {
    let extensions = ImageFormat::get_formats()
        .iter()
        .map(|x| {
            x.extensions_str().iter().map(move |y| SupportedFormatMeta {
                ext: <str as ToString>::to_string(*y),
                readable: ImageFormat::can_read(&x),
                writable: ImageFormat::can_write(&x),
            })
        })
        .flatten()
        .collect::<Vec<SupportedFormatMeta>>();
    Ok(extensions)
}

fn compress_file_handler(app: &tauri::AppHandle) {
    let emitter_name = "listen-file";
    let file_state = FileState::new(HashMap::new());
    let app_handle = app.app_handle();
    // listen file input
    let emit_file_create = app_handle
        .app_handle()
        .listen_global("emit-file", move |event| {
            // get task
            let task: EmitFileRequestBody = serde_json::from_str(&event.payload().unwrap())
                .expect("JSON was not well-formatted");

            match task.operation {
                // notify to client for ready
                EmitFileOperation::Create => {
                    // check
                    if let Some(v) = &task.files {
                        // process parallelly
                        v.into_par_iter().for_each(|(path, _)| {
                            let add_file = file_state.add_file(&path);
                            notify_file_to_client(&app_handle, &add_file, emitter_name);
                        });
                    };
                }
                // compress
                EmitFileOperation::Compress => {
                    // check
                    if let Some(v) = &task.files {

                        // wheather process parallelly or not
                        let paralell_processable =
                            v.into_par_iter().all(|(path, _)|
                            // process strategy with multithreading depends on it's library implementation
                            matches!(
                                ImageFormat::process_strategy(&ImageFormat::from_path(&path).unwrap()),
                                ProcessStrategy::Parallel
                            ));

                        match paralell_processable {
                            true => {
                                // process parallelly
                                v.clone()
                                    .into_par_iter()
                                    // skip compressed file
                                    .filter(|(_, file)| !matches!(file.status, Status::Success))
                                    .for_each(|(path, file)| {
                                        file_state.process_compress(
                                            &app_handle,
                                            &emitter_name,
                                            path,
                                            file,
                                        )
                                    });
                            }
                            _ => {
                                // process seriesly
                                v.clone()
                                    .into_iter()
                                    // skip compressed file
                                    .filter(|(_, file)| !matches!(file.status, Status::Success))
                                    .for_each(|(path, file)| {
                                        file_state.process_compress(
                                            &app_handle,
                                            &emitter_name,
                                            path,
                                            file,
                                        )
                                    });
                            }
                        };
                    };
                }
                EmitFileOperation::Update => {
                    file_state.update_options(task.options);
                }
                EmitFileOperation::Clear => {
                    file_state.clear();
                    notify_file_to_client(&app_handle, &EmitFileOperation::Clear, "listen-clear-file");
                }
            }
        });
}

fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(
                buf,
                "[{} {} {}] {} {}:{}",
                ts,
                record.level(),
                record.target(),
                record.args(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
            )
        })
        .init();
}

fn main() {
    #[cfg(debug_assertions)]
    init_logger();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_supported_extensions,
            get_compress_options_context,
        ])
        .setup(|app| {
            compress_file_handler(&app.app_handle());

            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
