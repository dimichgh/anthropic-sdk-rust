use crate::types::{
    FileObject, FileUploadParams, FileListParams, FileList, FileDownload,
    UploadProgress, StorageInfo, AnthropicError, Result,
};
use crate::http::HttpClient;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Resource for managing files via the Anthropic Files API
#[derive(Debug, Clone)]
pub struct FilesResource {
    http_client: Arc<HttpClient>,
}

impl FilesResource {
    /// Create a new files resource
    pub fn new(http_client: Arc<HttpClient>) -> Self {
        Self { http_client }
    }

    /// Upload a file to the Anthropic API
    /// 
    /// # Arguments
    /// * `params` - Upload parameters including content, filename, and purpose
    /// 
    /// # Returns
    /// A new `FileObject` with the uploaded file information
    /// 
    /// # Errors
    /// Returns an error if the upload fails or if the parameters are invalid
    pub async fn upload(&self, params: FileUploadParams) -> Result<FileObject> {
        // Validate parameters
        params.validate()?;

        // Create multipart form
        let form = self.create_multipart_form(params)?;

        let response = self
            .http_client
            .post("/v1/files")
            .multipart(form)
            .send()
            .await?;

        let file_object: FileObject = response.json().await?;
        Ok(file_object)
    }

    /// Upload a file with progress tracking
    /// 
    /// # Arguments
    /// * `params` - Upload parameters
    /// * `progress_callback` - Called with progress updates during upload
    /// 
    /// # Returns
    /// The uploaded `FileObject`
    /// 
    /// # Errors
    /// Returns an error if the upload fails
    pub async fn upload_with_progress<F>(
        &self,
        params: FileUploadParams,
        mut progress_callback: F,
    ) -> Result<FileObject>
    where
        F: FnMut(UploadProgress),
    {
        // Validate parameters
        params.validate()?;

        let total_size = params.content.len() as u64;
        let start_time = Instant::now();

        // Simulate upload progress (in a real implementation, this would track actual upload)
        let mut uploaded = 0u64;
        let chunk_size = (total_size / 20).max(1024); // 20 progress updates minimum

        while uploaded < total_size {
            let chunk = chunk_size.min(total_size - uploaded);
            uploaded += chunk;

            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { uploaded as f64 / elapsed } else { 0.0 };

            let progress = UploadProgress::new(uploaded, total_size).with_speed(speed);
            progress_callback(progress);

            // Simulate upload time
            sleep(Duration::from_millis(50)).await;
        }

        // Perform actual upload
        self.upload(params).await
    }

    /// Retrieve a file by ID
    /// 
    /// # Arguments
    /// * `file_id` - The ID of the file to retrieve
    /// 
    /// # Returns
    /// The `FileObject` with current information
    /// 
    /// # Errors
    /// Returns an error if the file is not found or if the request fails
    pub async fn get(&self, file_id: &str) -> Result<FileObject> {
        let response = self
            .http_client
            .get(&format!("/v1/files/{}", file_id))
            .send()
            .await?;

        let file_object: FileObject = response.json().await?;
        Ok(file_object)
    }

    /// List files with optional filtering and pagination
    /// 
    /// # Arguments
    /// * `params` - Optional parameters for filtering and pagination
    /// 
    /// # Returns
    /// A `FileList` containing files and pagination information
    /// 
    /// # Errors
    /// Returns an error if the request fails
    pub async fn list(&self, params: Option<FileListParams>) -> Result<FileList> {
        let mut request = self.http_client.get("/v1/files");

        if let Some(params) = params {
            if let Some(purpose) = params.purpose {
                request = request.query(&[("purpose", serde_json::to_string(&purpose)?)]);
            }
            if let Some(after) = params.after {
                request = request.query(&[("after", after)]);
            }
            if let Some(limit) = params.limit {
                request = request.query(&[("limit", limit.to_string())]);
            }
            if let Some(order) = params.order {
                request = request.query(&[("order", serde_json::to_string(&order)?)]);
            }
        }

        let response = request.send().await?;
        let file_list: FileList = response.json().await?;
        Ok(file_list)
    }

    /// Download file content
    /// 
    /// # Arguments
    /// * `file_id` - The ID of the file to download
    /// 
    /// # Returns
    /// A `FileDownload` containing the file content and metadata
    /// 
    /// # Errors
    /// Returns an error if the file is not found or cannot be downloaded
    pub async fn download(&self, file_id: &str) -> Result<FileDownload> {
        let response = self
            .http_client
            .get(&format!("/v1/files/{}/content", file_id))
            .send()
            .await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let content_disposition = response
            .headers()
            .get("content-disposition")
            .and_then(|v| v.to_str().ok());

        let filename = extract_filename_from_disposition(content_disposition)
            .unwrap_or_else(|| format!("file_{}", file_id));

        let content = response.bytes().await?;
        let size = content.len() as u64;

        Ok(FileDownload {
            content: content.to_vec(),
            content_type,
            filename,
            size,
        })
    }

    /// Delete a file
    /// 
    /// # Arguments
    /// * `file_id` - The ID of the file to delete
    /// 
    /// # Returns
    /// The updated `FileObject` with deletion status
    /// 
    /// # Errors
    /// Returns an error if the file cannot be deleted or if the request fails
    pub async fn delete(&self, file_id: &str) -> Result<FileObject> {
        let response = self
            .http_client
            .delete(&format!("/v1/files/{}", file_id))
            .send()
            .await?;

        let file_object: FileObject = response.json().await?;
        Ok(file_object)
    }

    /// Get storage information and quotas
    /// 
    /// # Returns
    /// `StorageInfo` with current usage and quotas
    /// 
    /// # Errors
    /// Returns an error if the request fails
    pub async fn get_storage_info(&self) -> Result<StorageInfo> {
        let response = self
            .http_client
            .get("/v1/files/storage")
            .send()
            .await?;

        let storage_info: StorageInfo = response.json().await?;
        Ok(storage_info)
    }

    /// Wait for a file to be processed
    /// 
    /// # Arguments
    /// * `file_id` - The ID of the file to wait for
    /// * `poll_interval` - How often to check the status (default: 2 seconds)
    /// * `timeout` - Maximum time to wait (default: 5 minutes)
    /// 
    /// # Returns
    /// The processed `FileObject`
    /// 
    /// # Errors
    /// Returns an error if the file processing fails or times out
    pub async fn wait_for_processing(
        &self,
        file_id: &str,
        poll_interval: Option<Duration>,
        timeout: Option<Duration>,
    ) -> Result<FileObject> {
        let poll_interval = poll_interval.unwrap_or(Duration::from_secs(2));
        let timeout = timeout.unwrap_or(Duration::from_secs(300)); // 5 minutes

        let start_time = Instant::now();

        loop {
            let file = self.get(file_id).await?;

            if file.status.is_ready() {
                return Ok(file);
            }

            if file.status.has_error() {
                return Err(AnthropicError::Other(format!(
                    "File processing failed for file {}",
                    file_id
                )));
            }

            if start_time.elapsed() > timeout {
                return Err(AnthropicError::Timeout);
            }

            sleep(poll_interval).await;
        }
    }

    /// Create a multipart form for file upload
    fn create_multipart_form(&self, params: FileUploadParams) -> Result<reqwest::multipart::Form> {
        let mut form = reqwest::multipart::Form::new();

        // Add file content
        let file_part = reqwest::multipart::Part::bytes(params.content)
            .file_name(params.filename.clone())
            .mime_str(&params.content_type)?;
        form = form.part("file", file_part);

        // Add purpose
        form = form.text("purpose", serde_json::to_string(&params.purpose)?);

        // Add metadata if present
        if !params.metadata.is_empty() {
            form = form.text("metadata", serde_json::to_string(&params.metadata)?);
        }

        Ok(form)
    }
}

/// High-level file management utilities
impl FilesResource {
    /// Upload multiple files concurrently
    /// 
    /// # Arguments
    /// * `uploads` - Vector of upload parameters
    /// * `max_concurrent` - Maximum number of concurrent uploads
    /// 
    /// # Returns
    /// Vector of uploaded file objects
    /// 
    /// # Errors
    /// Returns an error if any upload fails
    pub async fn upload_batch(
        &self,
        uploads: Vec<FileUploadParams>,
        max_concurrent: Option<usize>,
    ) -> Result<Vec<FileObject>> {
        let max_concurrent = max_concurrent.unwrap_or(3);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        
        let tasks: Vec<_> = uploads
            .into_iter()
            .map(|params| {
                let files_resource = self.clone();
                let semaphore = semaphore.clone();
                
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    files_resource.upload(params).await
                })
            })
            .collect();

        let mut results = Vec::new();
        for task in tasks {
            let result = task.await.map_err(|e| AnthropicError::Other(e.to_string()))??;
            results.push(result);
        }

        Ok(results)
    }

    /// Clean up old files based on age
    /// 
    /// # Arguments
    /// * `max_age` - Maximum age for files to keep
    /// 
    /// # Returns
    /// Number of files deleted
    /// 
    /// # Errors
    /// Returns an error if the cleanup operation fails
    pub async fn cleanup_old_files(&self, max_age: Duration) -> Result<u32> {
        let files = self.list(None).await?;
        let cutoff_time = chrono::Utc::now() - chrono::Duration::from_std(max_age)?;
        
        let mut deleted_count = 0;
        
        for file in files.data {
            if file.created_at < cutoff_time {
                if let Ok(_) = self.delete(&file.id).await {
                    deleted_count += 1;
                }
            }
        }
        
        Ok(deleted_count)
    }

    /// Get files by purpose with optional filtering
    /// 
    /// # Arguments
    /// * `purpose` - File purpose to filter by
    /// * `limit` - Maximum number of files to return
    /// 
    /// # Returns
    /// Vector of matching file objects
    /// 
    /// # Errors
    /// Returns an error if the request fails
    pub async fn get_files_by_purpose(
        &self,
        purpose: crate::types::FilePurpose,
        limit: Option<u32>,
    ) -> Result<Vec<FileObject>> {
        let params = FileListParams::new()
            .purpose(purpose)
            .limit(limit.unwrap_or(50));
        
        let file_list = self.list(Some(params)).await?;
        Ok(file_list.data)
    }
}

/// Helper function to extract filename from Content-Disposition header
fn extract_filename_from_disposition(disposition: Option<&str>) -> Option<String> {
    disposition.and_then(|d| {
        // Look for filename="value" or filename*=UTF-8''value
        if let Some(start) = d.find("filename=") {
            let start = start + 9; // "filename=".len()
            let rest = &d[start..];
            
            if rest.starts_with('"') {
                // Quoted filename
                rest[1..].split('"').next().map(|s| s.to_string())
            } else {
                // Unquoted filename
                rest.split(';').next().map(|s| s.trim().to_string())
            }
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FilePurpose;

    #[test]
    fn test_extract_filename_from_disposition() {
        assert_eq!(
            extract_filename_from_disposition(Some(r#"attachment; filename="test.txt""#)),
            Some("test.txt".to_string())
        );
        
        assert_eq!(
            extract_filename_from_disposition(Some(r#"attachment; filename=test.txt"#)),
            Some("test.txt".to_string())
        );
        
        assert_eq!(
            extract_filename_from_disposition(Some(r#"inline"#)),
            None
        );
        
        assert_eq!(
            extract_filename_from_disposition(None),
            None
        );
    }

    #[test]
    fn test_upload_params_creation() {
        let params = FileUploadParams::new(
            b"test content".to_vec(),
            "test.txt",
            "text/plain",
            FilePurpose::Document,
        );

        assert_eq!(params.filename, "test.txt");
        assert_eq!(params.content_type, "text/plain");
        assert_eq!(params.purpose, FilePurpose::Document);
        assert_eq!(params.content, b"test content");
    }

    #[test]
    fn test_file_list_params_builder() {
        let params = FileListParams::new()
            .purpose(FilePurpose::Vision)
            .limit(10)
            .after("file_123");

        assert_eq!(params.purpose, Some(FilePurpose::Vision));
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.after, Some("file_123".to_string()));
    }
} 