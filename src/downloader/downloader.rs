use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
use futures::StreamExt;
use tokio::sync::{Semaphore, TryAcquireError};
use std::{fs, thread};
use std::time::Duration;

use log::{debug, error, info};

#[derive(Clone, PartialEq)]
pub enum DownloadState {
    /// total
    Queued(u64),
    /// downloaded, total
    Downloading(u64, u64),
    Paused,
    /// size
    Completed(u64),
    Error(String),
    Cancelled,
}

pub enum QueueCommand {
    /// 添加下载任务 参数为(url, path)
    Add(String, String),
    /// 未实现
    Pause(u64),
    /// 未实现
    Resume(u64),
    /// 取消下载任务 参数为任务id
    Cancel(u64),
}

pub enum TaskCommand {
    Pause,
    Resume,
    Cancel,
}

pub struct DownloadTask {
    /// command sender
    pub controller: mpsc::Sender<TaskCommand>,
    pub handle: tokio::task::JoinHandle<()>,
    /// 从1开始
    pub id: u64,
    pub state: Arc<Mutex<DownloadState>>,
    pub url: String,
}

pub struct Downloader {
    client: reqwest::Client,
    runtime: Arc<tokio::runtime::Runtime>,
    tasks: Arc<Mutex<Vec<DownloadTask>>>,
    sender: mpsc::Sender<QueueCommand>,
}

/// 下载任务
impl DownloadTask {
    pub fn new(id: u64, url: String, path: String, client: reqwest::Client, rt: Arc<tokio::runtime::Runtime>, semaphore: Arc<Semaphore>) -> Self {
        info!("create id={id} url={url} path={path}");
        
        // 控制download task
        let (control_sender, control_receiver) = mpsc::channel();

        let state = Arc::new(Mutex::new(DownloadState::Queued(0)));

        let url_clone = url.clone();
        let state_clone = state.clone();
        let handle = rt.spawn(async move {
            let state = state_clone;

            let response = match client.get(&url).send().await {
                Ok(res) => res,
                Err(e) => {
                    error!("Failed to get response. Reason: {e}");
                    match state.lock() {
                        Ok(mut state) => *state = DownloadState::Error(String::from("Failed to download.")),
                        Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
                    }
                    return;
                }
            };

            let total_size = response.content_length().unwrap_or(0);

            match state.lock() {
                Ok(mut state) => *state = DownloadState::Queued(total_size),
                Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
            }

            while let Err(e) = semaphore.try_acquire() {
                if e == TryAcquireError::Closed {
                    error!("Acquire error: {e}");
                    match state.lock() {
                        Ok(mut state) => *state = DownloadState::Error(format!("{e}")),
                        Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
                    }
                    return;
                }

                if let Ok(cmd) = control_receiver.try_recv() {
                    match state.lock() {
                        Ok(mut state) => {
                            match cmd {
                                TaskCommand::Pause => {
                                    *state = DownloadState::Paused;
                                },
                                TaskCommand::Resume => {
                                    *state = DownloadState::Downloading(0, total_size);
                                },
                                TaskCommand::Cancel => {
                                    *state = DownloadState::Cancelled;
                                    return;
                                },
                            }
                        },
                        Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
                    }
                }
            }

            let mut stream = response.bytes_stream();            

            let mut file = match fs::File::create(&path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to create {path}. Reason: {e}");
                    match state.lock() {
                        Ok(mut state) => *state = DownloadState::Error(format!("{e}")),
                        Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
                    }
                    return;
                }
            };

            let mut downloaded: u64 = 0;

            info!("Start downloading {url}");

            let mut attempts = 0;

            while let Some(chunk) = stream.next().await {
                if let Ok(cmd) = control_receiver.try_recv() {
                    match state.lock() {
                        Ok(mut state) => {
                            match cmd {
                                TaskCommand::Pause => {
                                    *state = DownloadState::Paused;
                                },
                                TaskCommand::Resume => {
                                    *state = DownloadState::Downloading(0, total_size);
                                },
                                TaskCommand::Cancel => {
                                    *state = DownloadState::Cancelled;
                                    return;
                                },
                            }
                        },
                        Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
                    }
                }

                match state.lock() {
                    Ok(state) => {
                        if *state == DownloadState::Paused {
                            continue;
                        }
                    },
                    Err(e) => {
                        error!("Failed to lock a mutex. Reason: {e}");
                        continue;
                    },
                }

                match chunk {
                    Ok(chunk) => {
                        attempts = 0;

                        if let Err(e) = file.write_all(&chunk) {
                            error!("Failed to write chunk. Reason: {e}");
                            drop(file);
                            fs::remove_file(path).unwrap();
                            match state.lock() {
                                Ok(mut state) => *state = DownloadState::Error(format!("{e}")),
                                Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
                            }
                            return;
                        }
                        downloaded += chunk.len() as u64;
                        match state.lock() {
                            Ok(mut state) => *state = DownloadState::Downloading(downloaded, total_size),
                            Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
                        }
                    }
                    Err(e) => {
                        if attempts < 3 {
                            attempts += 1;
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            let range_header_value = format!("bytes={}-", downloaded);
                            let new_response = client.get(&url)
                                .header(reqwest::header::RANGE, range_header_value)
                                .send().await;
                            match new_response {
                                Ok(resp) => {
                                    stream = resp.bytes_stream();
                                    continue;
                                },
                                Err(e) => {
                                    error!("Failed to send request. Reason: {e}");
                                }
                            }
                        }

                        drop(file);
                        fs::remove_file(path).unwrap();
                        match state.lock() {
                            Ok(mut state) => *state = DownloadState::Error(format!("{e}")),
                            Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
                        }
                        return;
                    }
                }
            }

            match state.lock() {
                Ok(mut state) => *state = DownloadState::Completed(total_size),
                Err(e) => error!("Failed to lock a mutex. Reason: {e}"),
            }

            info!("Finish downloading {url}");
        });

        DownloadTask {
            controller: control_sender,
            handle,
            id,
            url: url_clone,
            state,
        }
    }
}

/// 下载器
impl Downloader {
    pub fn new(concurrency: usize) -> Self {
        let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap();
        let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let (sender, receiver) = mpsc::channel();
        let tasks = Arc::new(Mutex::new(Vec::new()));

        let client_clone = client.clone();
        let rt_clone = rt.clone();
        let tasks_clone = tasks.clone();

        // 命令线程
        thread::spawn(move || {
            let client = client_clone;
            let rt = rt_clone;
            let tasks = tasks_clone;
            
            let semaphore = Arc::new(Semaphore::new(concurrency));

            let _tokio = rt.enter();

            while let Ok(cmd) = receiver.recv() {
                match cmd {
                    QueueCommand::Add(url, path) => {
                        if let Ok(mut tasks) = tasks.lock() {
                            let id = (tasks.len() + 1) as u64;
                            tasks.push(DownloadTask::new(id, url, path, client.clone(), rt.clone(), semaphore.clone()));
                        } else {
                            error!("Command thread: Failed to lock tasks");
                        }
                    },
                    QueueCommand::Cancel(id) => {
                        if let Ok(mut tasks) = tasks.lock() {
                            if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
                                task.controller.send(TaskCommand::Cancel).unwrap();
                            }
                        } else {
                            error!("Command thread: Failed to lock tasks");
                        }
                    },
                    _ => {
                        debug!("Not implemented.");
                    },
                }
            }

            error!("Command sender is dropped.");
        });

        Downloader { client, runtime: rt, tasks, sender }
    }

    pub fn add(&self, url: String, path: String) -> Result<(), mpsc::SendError<QueueCommand>> {
        match self.sender.send(QueueCommand::Add(url, path)) {
            Ok(_) => { Ok(()) }
            Err(e) => {
                error!("Failed to send a command. Reason: {e}");
                Err(e)
            }
        }
    }

    pub fn cancel(&self, id: u64) -> Result<(), mpsc::SendError<QueueCommand>> {
        self.sender.send(QueueCommand::Cancel(id))
    }

    pub fn clear(&self) -> Option<()> {
        if let Ok(mut tasks) = self.tasks.lock() {
            for task in tasks.iter() {
                if let Err(e) = task.controller.send(TaskCommand::Cancel) {
                    error!("Failed to send a command. Reason: {e}");
                    return None;
                }
            }
            tasks.clear();
            Some(())
        } else {
            None
        }
    }

    /// returns Vec<(id, url, state)>
    pub fn get_tasks(&self) -> Option<Vec<(u64, String, DownloadState)>> {
        if let Ok(tasks) = self.tasks.lock() {
            let mut result = Vec::new();

            for task in tasks.iter() {
                if let Ok(state) = task.state.lock() {
                    result.push((task.id, task.url.clone(), state.clone()));
                }
            }

            Some(result)
        } else {
            None
        }
    }

    pub fn has_error(&self) -> bool {
        if let Some(tasks) = self.get_tasks() {
            for (_, _, state) in tasks {
                match state {
                    DownloadState::Error(_) => { return true; }
                    _ => {}
                }
            }

            return false;
        } else {
            return true;
        }
    }

    pub fn in_progress(&self) -> bool {
        if let Some(tasks) = self.get_tasks() {
            for (_, _, state) in tasks {
                match state {
                    DownloadState::Paused => { return true; }
                    DownloadState::Queued(_) => { return true; }
                    DownloadState::Downloading(_, _) => { return true; }
                    _ => {}
                }
            }

            return false;
        } else {
            return true;
        }
    }

    pub fn update_progress(&self, f: impl Fn(f64) -> () + 'static + Send) -> thread::JoinHandle<()> {
        let tasks = self.tasks.clone();
        thread::spawn(move || {
            loop {
                if let Ok(tasks) = tasks.lock() {
                    let mut downloaded = 0.0;
                    let mut total = 0.0;
    
                    for task in tasks.iter() {
                        if let Ok(state) = task.state.lock() {
                            match *state {
                                DownloadState::Completed(size) => {
                                    downloaded += size as f64;
                                    total += size as f64;
                                },
                                DownloadState::Downloading(downloaded_size, total_size) => {
                                    downloaded += downloaded_size as f64;
                                    total += total_size as f64;
                                },
                                // DownloadState::Paused => {
                                // }
                                DownloadState::Queued(size) => {
                                    total += size as f64;
                                }
                                _ => {},
                            }
                        }
                    }

                    f(downloaded / total);
                } else {
                    error!("State thread: Failed to lock tasks.");
                }
    
                sleep(Duration::from_millis(500));
            }
        })
    }
}

impl Default for Downloader {
    fn default() -> Self {
        let (sender, _) = mpsc::channel();
        Downloader {
            client: reqwest::Client::new(),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
            tasks: Arc::new(Mutex::new(Vec::new())),
            sender,
        }
    }
}
