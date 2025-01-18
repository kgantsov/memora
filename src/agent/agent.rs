use crate::schema::file::{
    FileCreateRequest, FileResponse, FileStatus, FileType, FileUpdateRequest,
};
use aws_config::default_provider::token;
use charybdis::types::Uuid;

use clap::Parser;
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

pub struct Agent {
    token: String,
    scan_dir: PathBuf,
    scan_interval: u64,

    semaphore: Arc<Semaphore>,

    // Reqwest HTTP client
    client: Arc<Client>,
}

impl Agent {
    pub fn new(token: String, scan_dir: PathBuf) -> Self {
        // Create a semaphore to limit the number of concurrent workers
        let max_workers = 4; // Set the number of workers
        let semaphore = Arc::new(Semaphore::new(max_workers));
        let client = Arc::new(Client::new()); // Shared HTTP client for uploads

        Self {
            token,
            scan_dir,
            scan_interval: 5,
            semaphore,
            client,
        }
    }

    // run_scanner is periodically scans a file system for changes
    pub async fn run_scanner(&self) {
        println!("Running scanner with token: {}", self.token);

        // timer to run scanner every self.scan_interval seconds
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(self.scan_interval));

        loop {
            interval.tick().await;
            println!("Scanner tick");
            self.scan_dir().await.unwrap();
        }
    }

    pub async fn scan_dir(&self) -> Result<(), std::io::Error> {
        let mut stack = vec![self.scan_dir.clone()]; // Stack to manage directories to visit
        let mut tasks = Vec::new();

        // let path = Path::new(self.scan_dir.as_path());

        // Iterate while there are directories to process
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Add the subdirectory to the stack for later processing
                    stack.push(path.clone());
                    Self::create_file(
                        self.token.clone(),
                        &path,
                        FileType::DIRECTORY,
                        self.client.clone(),
                    )
                    .await?;
                } else if path.is_file() {
                    // Upload the file in parallel, limited by the semaphore
                    let path_clone = path.clone();
                    let permit = self.semaphore.clone().acquire_owned().await.unwrap();
                    let client_clone = self.client.clone();
                    let token_clone = self.token.clone();

                    let task = task::spawn(async move {
                        Self::upload_file(token_clone, &path_clone, client_clone).await;
                        drop(permit); // Release the semaphore permit
                    });
                    tasks.push(task);
                }
            }
        }

        // Await all tasks
        // join_all(tasks).await;
        for task in tasks {
            task.await.unwrap(); // Wait for all tasks to complete
        }
        Ok(())
    }

    async fn create_file(
        token: String,
        path: &Path,
        file_type: FileType,
        client: Arc<Client>,
    ) -> Result<FileResponse, std::io::Error> {
        let data = FileCreateRequest {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            directory: path.parent().unwrap().to_string_lossy().to_string(),
            file_type,
            status: FileStatus::OPEN,
        };

        println!("Creating: {:?}", data);

        let res = client
            .post("http://localhost:8000/v1/files")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("bearer {}", token))
            .json(&data)
            .send()
            .await;

        match res {
            Ok(response) if response.status().is_success() => {
                println!("Created: {}", path.file_name().unwrap().to_string_lossy());

                match response.json::<FileResponse>().await {
                    Ok(file) => Ok(file),
                    Err(e) => {
                        eprintln!("Error parsing response: {}", e);
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        ))
                    }
                }
            }
            Ok(response) => {
                eprintln!(
                    "Failed to create {}: HTTP {}",
                    path.file_name().unwrap().to_string_lossy(),
                    response.status()
                );
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to create file",
                ))
            }
            Err(e) => {
                eprintln!(
                    "Error creating {}: {}",
                    path.file_name().unwrap().to_string_lossy(),
                    e
                );
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            }
        }
    }

    async fn update_file(
        token: String,
        file_id: Uuid,
        data: FileUpdateRequest,
        client: Arc<Client>,
    ) -> Result<FileResponse, std::io::Error> {
        let res = client
            .put(format!("http://localhost:8000/v1/files/{}", file_id))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("bearer {}", token))
            .json(&data)
            .send()
            .await;

        match res {
            Ok(response) if response.status().is_success() => {
                println!("Updated: {:?}", data);

                match response.json::<FileResponse>().await {
                    Ok(file) => Ok(file),
                    Err(e) => {
                        eprintln!("Error parsing response: {}", e);
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        ))
                    }
                }
            }
            Ok(response) => {
                eprintln!("Failed to update {:?}: HTTP {}", data, response.status());
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to update file",
                ))
            }
            Err(e) => {
                eprintln!("Error updating {:?}: {}", data, e);
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            }
        }
    }

    // Function to upload a file to the server
    async fn upload_file(
        token: String,
        path: &Path,
        client: Arc<Client>,
    ) -> Result<(), std::io::Error> {
        let file_name = path.file_name().unwrap().to_string_lossy();
        println!("Uploading: {}", file_name);
        let file = Self::create_file(token.clone(), &path, FileType::FILE, client.clone()).await?;

        match file.upload_presigned_url {
            Some(url) => {
                // Simulate a file upload by sending a POST request
                let result = client
                    .put(url)
                    .body(fs::read(path).unwrap_or_default()) // Read the file content
                    .send()
                    .await;

                match result {
                    Ok(response) if response.status().is_success() => {
                        println!("Uploaded: {}", file_name);

                        Self::update_file(
                            token,
                            file.id,
                            FileUpdateRequest {
                                name: file.name,
                                directory: file.directory,
                                file_type: FileType::FILE,
                                status: FileStatus::CLOSED,
                                modified_at: file.modified_at,
                                created_at: file.created_at,
                            },
                            client.clone(),
                        )
                        .await?;

                        Ok(())
                    }
                    Ok(response) => {
                        eprintln!("Failed to upload {}: HTTP {}", file_name, response.status());
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Error uploading {}: {}", file_name, e);
                        Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                    }
                }
            }
            None => {
                eprintln!("Error: No upload URL found for {}", file_name);
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No upload URL found",
                ))
            }
        }
    }
}
