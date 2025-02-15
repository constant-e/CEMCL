use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
use futures::StreamExt;
use tokio::sync::Semaphore;
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
    /// progress receiver
    pub progress_receiver: mpsc::Receiver<DownloadState>,
    pub state: DownloadState,
    pub url: String,
}

pub struct Downloader {
    client: reqwest::Client,
    tasks: Arc<Mutex<Vec<DownloadTask>>>,
    sender: mpsc::Sender<QueueCommand>,
}

/// 下载任务
impl DownloadTask {
    pub fn new(id: u64, url: String, path: String, client: reqwest::Client, rt: Arc<tokio::runtime::Runtime>, semaphore: Arc<Semaphore>) -> Self {
        info!("create id={id} url={url} path={path}");
        
        // 控制download task
        let (control_sender, control_receiver) = mpsc::channel();
        // 接收状态
        let (progress_sender, progress_receiver) = mpsc::channel();

        let url_clone = url.clone();
        let handle = rt.spawn(async move {
            let response = match client.get(&url).send().await {
                Ok(res) => res,
                Err(e) => {
                    error!("Failed to get response. Reason: {e}");
                    progress_sender.send(DownloadState::Error(String::from("Failed to download."))).unwrap();
                    return;
                }
            };

            let total_size = response.content_length().unwrap_or(0);
            let mut stream = response.bytes_stream();            

            let mut file = fs::File::create(path).unwrap();
            let mut downloaded: u64 = 0;

            let mut state = DownloadState::Queued(total_size);
            progress_sender.send(state.clone()).unwrap();

            if let Err(e) = semaphore.acquire().await {
                error!("Acquire error: {e}");
                state = DownloadState::Error(format!("{e}"));
                progress_sender.send(state).unwrap();
                return;
            }

            info!("Start downloading {url}");

            while let Some(chunk) = stream.next().await {
                let state = &mut state;
                if let Ok(cmd) = control_receiver.try_recv() {
                    match cmd {
                        TaskCommand::Pause => {
                            *state = DownloadState::Paused;
                        },
                        TaskCommand::Resume => {
                            *state = DownloadState::Downloading(downloaded, total_size);
                        },
                        TaskCommand::Cancel => {
                            *state = DownloadState::Cancelled;
                            break;
                        },
                    }
                    progress_sender.send(state.clone()).unwrap();
                }

                if *state == DownloadState::Paused {
                    continue;
                }

                match chunk {
                    Ok(chunk) => {
                        if let Err(e) = file.write_all(&chunk) {
                            error!("Failed to write chunk. Reason: {e}");
                            *state = DownloadState::Error(format!("{e}"));
                        }
                        downloaded += chunk.len() as u64;
                        *state = DownloadState::Downloading(downloaded, total_size);
                        
                    }
                    Err(e) => {
                        *state = DownloadState::Error(format!("{e}"));
                    }
                }
                
                progress_sender.send(state.clone()).unwrap();
            }
            state = DownloadState::Completed(total_size);

            progress_sender.send(state).unwrap();

            info!("Finish downloading {url}");
        });

        DownloadTask {
            controller: control_sender,
            handle,
            id,
            url: url_clone,
            progress_receiver,
            state: DownloadState::Queued(0),
        }
    }
}

/// 下载器
impl Downloader {
    pub fn new(app_ui_weak: slint::Weak<crate::AppWindow>, concurrency: usize) -> Self {
        let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap();
        let (sender, receiver) = mpsc::channel();
        let tasks = Arc::new(Mutex::new(Vec::new()));

        let client_clone = client.clone();
        let tasks_clone = tasks.clone();

        // 命令线程
        thread::spawn(move || {
            let client = client_clone;
            let tasks = tasks_clone;

            let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
            let _tokio = rt.enter();
            let semaphore = Arc::new(Semaphore::new(concurrency));

            loop {
                if let Ok(cmd) = receiver.recv() {
                    match cmd {
                        QueueCommand::Add(url, path) => {
                            if let Ok(mut tasks) = tasks.try_lock() {
                                let id = (tasks.len() + 1) as u64;
                                tasks.push(DownloadTask::new(id, url, path, client.clone(), rt.clone(), semaphore.clone()));
                            } else {
                                error!("Failed to lock tasks");
                            }
                        },
                        QueueCommand::Cancel(id) => {
                            if let Ok(mut q) = tasks.try_lock() {
                                if let Some(task) = q.iter_mut().find(|t| t.id == id) {
                                    task.controller.send(TaskCommand::Cancel).unwrap();
                                }
                            }
                        },
                        _ => {
                            debug!("Not implemented.");
                        },
                    }
                } else {
                    error!("Failed to receive a command.");
                    break;
                }
            }
        });

        let tasks_clone = tasks.clone();
        // 监听状态
        thread::spawn(move || {
            loop {
                if let Ok(mut tasks) = tasks_clone.lock() {
                    
                    let mut downloaded = 0.0;
                    let mut total = 0.0;
                    let mut in_progress = false;

                    for task in tasks.iter_mut() {
                        if let Ok(state) = task.progress_receiver.try_recv() {
                            task.state = state;
                            match task.state {
                                DownloadState::Completed(size) => {
                                    downloaded += size as f64;
                                    total += size as f64;
                                },
                                DownloadState::Downloading(downloaded_size, total_size) => {
                                    in_progress = true;
                                    downloaded += downloaded_size as f64;
                                    total += total_size as f64;
                                },
                                DownloadState::Paused => {
                                    in_progress = true;
                                }
                                DownloadState::Queued(size) => {
                                    in_progress = true;
                                    total += size as f64;
                                }
                                _ => {},
                            }
                        }
                    }

                    if in_progress {
                        app_ui_weak.upgrade_in_event_loop(move |ui| {
                            ui.set_progress((downloaded / total) as f32);
                        });
                    }
                    
                } else {
                    error!("Failed to lock a mutex.")
                }

                sleep(Duration::from_millis(50));
            }
        });

        Downloader { client, tasks, sender }
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
        if let Ok(mut tasks) = self.tasks.try_lock() {
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
                result.push((task.id, task.url.clone(), task.state.clone()));
            }

            Some(result)
        } else {
            None
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
}

impl Default for Downloader {
    fn default() -> Self {
        let (sender, _) = mpsc::channel();
        Downloader {
            client: reqwest::Client::new(),
            tasks: Arc::new(Mutex::new(Vec::new())),
            sender,
        }
    }
}
