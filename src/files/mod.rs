use std::io::{Read, BufReader};
use std::path::Path;
use std::fs::File as StdFile;
use std::fmt;
use bytes::Bytes;
use mime::Mime;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};

/// Represents a file that can be uploaded to the Anthropic API
#[derive(Debug, Clone)]
pub struct File {
    /// File name
    pub name: String,
    /// MIME type
    pub mime_type: Mime,
    /// File data
    pub data: FileData,
    /// File size in bytes
    pub size: u64,
    /// Optional file hash for integrity verification
    pub hash: Option<String>,
}

/// Different sources of file data
#[derive(Debug, Clone)]
pub enum FileData {
    /// In-memory bytes
    Bytes(Bytes),
    /// Base64 encoded data
    Base64(String),
    /// File path for lazy loading
    Path(std::path::PathBuf),
    /// Temporary file
    TempFile(std::path::PathBuf),
}

/// File validation constraints
#[derive(Debug, Clone)]
pub struct FileConstraints {
    /// Maximum file size in bytes (default: 10MB)
    pub max_size: u64,
    /// Allowed MIME types (None = allow all)
    pub allowed_types: Option<Vec<Mime>>,
    /// Require hash verification
    pub require_hash: bool,
}

impl Default for FileConstraints {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024, // 10MB
            allowed_types: None,
            require_hash: false,
        }
    }
}

/// Errors that can occur during file operations
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("File not found: {path}")]
    NotFound { path: String },
    
    #[error("File too large: {size} bytes (max: {max_size} bytes)")]
    TooLarge { size: u64, max_size: u64 },
    
    #[error("Invalid MIME type: {mime_type} (allowed: {allowed:?})")]
    InvalidMimeType { mime_type: String, allowed: Vec<String> },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid base64 data: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
    
    #[error("MIME detection failed")]
    MimeDetectionFailed,
    
    #[error("Hash verification failed")]
    HashVerificationFailed,
    
    #[error("Invalid file data")]
    InvalidData,
}

impl File {
    /// Create a new file from bytes
    pub fn from_bytes(
        name: impl Into<String>,
        bytes: impl Into<Bytes>,
        mime_type: Option<Mime>,
    ) -> Result<Self, FileError> {
        let name = name.into();
        let bytes = bytes.into();
        let size = bytes.len() as u64;
        
        let mime_type = match mime_type {
            Some(mime) => mime,
            None => detect_mime_type(&name, Some(&bytes))?,
        };

        Ok(Self {
            name,
            mime_type,
            data: FileData::Bytes(bytes),
            size,
            hash: None,
        })
    }

    /// Create a new file from a file path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(FileError::NotFound {
                path: path.display().to_string(),
            });
        }
        
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();
            
        let mime_type = detect_mime_type(&name, None)?;

        Ok(Self {
            name,
            mime_type,
            data: FileData::Path(path.to_path_buf()),
            size,
            hash: None,
        })
    }

    /// Create a new file from base64 data
    pub fn from_base64(
        name: impl Into<String>,
        base64_data: impl Into<String>,
        mime_type: Option<Mime>,
    ) -> Result<Self, FileError> {
        let name = name.into();
        let base64_data = base64_data.into();
        
        // Decode to get size
        let decoded = general_purpose::STANDARD.decode(&base64_data)?;
        let size = decoded.len() as u64;
        
        let mime_type = match mime_type {
            Some(mime) => mime,
            None => detect_mime_type(&name, Some(&decoded))?,
        };

        Ok(Self {
            name,
            mime_type,
            data: FileData::Base64(base64_data),
            size,
            hash: None,
        })
    }

    /// Create a file from a standard library File
    pub fn from_std_file(
        std_file: StdFile,
        name: impl Into<String>,
        mime_type: Option<Mime>,
    ) -> Result<Self, FileError> {
        let name = name.into();
        let metadata = std_file.metadata()?;
        let size = metadata.len();
        
        // Read file contents
        let mut reader = BufReader::new(std_file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        
        let mime_type = match mime_type {
            Some(mime) => mime,
            None => detect_mime_type(&name, Some(&buffer))?,
        };

        Ok(Self {
            name,
            mime_type,
            data: FileData::Bytes(Bytes::from(buffer)),
            size,
            hash: None,
        })
    }

    /// Validate the file against constraints
    pub fn validate(&self, constraints: &FileConstraints) -> Result<(), FileError> {
        // Check size
        if self.size > constraints.max_size {
            return Err(FileError::TooLarge {
                size: self.size,
                max_size: constraints.max_size,
            });
        }

        // Check MIME type
        if let Some(allowed_types) = &constraints.allowed_types {
            if !allowed_types.iter().any(|mime| mime == &self.mime_type) {
                return Err(FileError::InvalidMimeType {
                    mime_type: self.mime_type.to_string(),
                    allowed: allowed_types.iter().map(|m| m.to_string()).collect(),
                });
            }
        }

        Ok(())
    }

    /// Get the file data as bytes
    pub async fn to_bytes(&self) -> Result<Bytes, FileError> {
        match &self.data {
            FileData::Bytes(bytes) => Ok(bytes.clone()),
            FileData::Base64(base64_data) => {
                let decoded = general_purpose::STANDARD.decode(base64_data)?;
                Ok(Bytes::from(decoded))
            },
            FileData::Path(path) => {
                let bytes = tokio::fs::read(path).await?;
                Ok(Bytes::from(bytes))
            },
            FileData::TempFile(path) => {
                let bytes = tokio::fs::read(path).await?;
                Ok(Bytes::from(bytes))
            },
        }
    }

    /// Get the file data as base64 string
    pub async fn to_base64(&self) -> Result<String, FileError> {
        match &self.data {
            FileData::Base64(base64_data) => Ok(base64_data.clone()),
            _ => {
                let bytes = self.to_bytes().await?;
                Ok(general_purpose::STANDARD.encode(&bytes))
            }
        }
    }

    /// Calculate and set the file hash
    pub async fn calculate_hash(&mut self) -> Result<String, FileError> {
        let bytes = self.to_bytes().await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());
        self.hash = Some(hash.clone());
        Ok(hash)
    }

    /// Verify the file hash
    pub async fn verify_hash(&self, expected_hash: &str) -> Result<bool, FileError> {
        let bytes = self.to_bytes().await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_hash = format!("{:x}", hasher.finalize());
        Ok(actual_hash == expected_hash)
    }

    /// Check if this is an image file
    pub fn is_image(&self) -> bool {
        self.mime_type.type_() == mime::IMAGE
    }

    /// Check if this is a text file
    pub fn is_text(&self) -> bool {
        self.mime_type.type_() == mime::TEXT
    }

    /// Check if this is an application file (e.g., PDF, document)
    pub fn is_application(&self) -> bool {
        self.mime_type.type_() == mime::APPLICATION
    }
}

/// Utility function to create a File from various sources (like TypeScript SDK's toFile)
pub async fn to_file(
    source: FileSource,
    name: Option<String>,
    mime_type: Option<Mime>,
) -> Result<File, FileError> {
    match source {
        FileSource::Bytes(bytes) => {
            let name = name.unwrap_or_else(|| "file".to_string());
            File::from_bytes(name, bytes, mime_type)
        },
        FileSource::Base64(base64_data) => {
            let name = name.unwrap_or_else(|| "file".to_string());
            File::from_base64(name, base64_data, mime_type)
        },
        FileSource::Path(path) => File::from_path(path),
        FileSource::StdFile(std_file, file_name) => {
            let name = name.or(file_name).unwrap_or_else(|| "file".to_string());
            File::from_std_file(std_file, name, mime_type)
        },
    }
}

/// Different sources for creating files
pub enum FileSource {
    /// Raw bytes
    Bytes(Bytes),
    /// Base64 encoded string
    Base64(String),
    /// File system path
    Path(std::path::PathBuf),
    /// Standard library File with optional name
    StdFile(StdFile, Option<String>),
}

/// Detect MIME type from filename and optional file data
fn detect_mime_type(filename: &str, data: Option<&[u8]>) -> Result<Mime, FileError> {
    // First try to detect from file extension
    if let Some(extension) = Path::new(filename).extension() {
        if let Some(ext_str) = extension.to_str() {
            let mime_type = match ext_str.to_lowercase().as_str() {
                // Images
                "jpg" | "jpeg" => mime::IMAGE_JPEG,
                "png" => mime::IMAGE_PNG,
                "gif" => mime::IMAGE_GIF,
                "webp" => "image/webp".parse().unwrap(),
                "svg" => mime::IMAGE_SVG,
                "bmp" => "image/bmp".parse().unwrap(),
                
                // Documents
                "pdf" => "application/pdf".parse().unwrap(),
                "doc" => "application/msword".parse().unwrap(),
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".parse().unwrap(),
                "txt" => mime::TEXT_PLAIN,
                "md" => "text/markdown".parse().unwrap(),
                "rtf" => "application/rtf".parse().unwrap(),
                
                // Spreadsheets
                "xls" => "application/vnd.ms-excel".parse().unwrap(),
                "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".parse().unwrap(),
                
                // Presentations
                "ppt" => "application/vnd.ms-powerpoint".parse().unwrap(),
                "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation".parse().unwrap(),
                
                // Audio
                "mp3" => "audio/mpeg".parse().unwrap(),
                "wav" => "audio/wav".parse().unwrap(),
                "ogg" => "audio/ogg".parse().unwrap(),
                
                // Video
                "mp4" => "video/mp4".parse().unwrap(),
                "avi" => "video/x-msvideo".parse().unwrap(),
                "mov" => "video/quicktime".parse().unwrap(),
                
                // Archives
                "zip" => "application/zip".parse().unwrap(),
                "tar" => "application/x-tar".parse().unwrap(),
                "gz" => "application/gzip".parse().unwrap(),
                
                // JSON/XML
                "json" => mime::APPLICATION_JSON,
                "xml" => mime::TEXT_XML,
                
                _ => mime::APPLICATION_OCTET_STREAM,
            };
            return Ok(mime_type);
        }
    }

    // If no extension, try magic bytes detection
    if let Some(bytes) = data {
        if bytes.len() >= 4 {
            let magic = &bytes[0..4];
            
            // PNG magic bytes
            if magic == [0x89, 0x50, 0x4E, 0x47] {
                return Ok(mime::IMAGE_PNG);
            }
            
            // JPEG magic bytes
            if magic[0..2] == [0xFF, 0xD8] {
                return Ok(mime::IMAGE_JPEG);
            }
            
            // PDF magic bytes
            if magic == [0x25, 0x50, 0x44, 0x46] {
                return Ok("application/pdf".parse().unwrap());
            }
            
            // GIF magic bytes
            if magic[0..3] == [0x47, 0x49, 0x46] {
                return Ok(mime::IMAGE_GIF);
            }
        }
    }

    // Default fallback
    Ok(mime::APPLICATION_OCTET_STREAM)
}

/// File upload builder for complex scenarios
#[derive(Debug)]
pub struct FileBuilder {
    name: Option<String>,
    mime_type: Option<Mime>,
    constraints: FileConstraints,
    calculate_hash: bool,
}

impl FileBuilder {
    /// Create a new file builder
    pub fn new() -> Self {
        Self {
            name: None,
            mime_type: None,
            constraints: FileConstraints::default(),
            calculate_hash: false,
        }
    }

    /// Set the file name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the MIME type
    pub fn mime_type(mut self, mime_type: Mime) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Set file constraints
    pub fn constraints(mut self, constraints: FileConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Enable hash calculation
    pub fn with_hash(mut self) -> Self {
        self.calculate_hash = true;
        self
    }

    /// Build file from source
    pub async fn build(self, source: FileSource) -> Result<File, FileError> {
        let mut file = to_file(source, self.name, self.mime_type).await?;
        
        // Validate constraints
        file.validate(&self.constraints)?;
        
        // Calculate hash if requested
        if self.calculate_hash {
            file.calculate_hash().await?;
        }
        
        Ok(file)
    }
}

impl Default for FileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "File {{ name: {}, type: {}, size: {} bytes }}",
            self.name, self.mime_type, self.size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn test_file_from_bytes() {
        let data = b"Hello, world!";
        let file = File::from_bytes("test.txt", Bytes::from_static(data), None).unwrap();
        
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.size, 13);
        assert_eq!(file.mime_type, mime::TEXT_PLAIN);
    }

    #[test]
    fn test_mime_detection() {
        assert_eq!(detect_mime_type("test.jpg", None).unwrap(), mime::IMAGE_JPEG);
        assert_eq!(detect_mime_type("test.png", None).unwrap(), mime::IMAGE_PNG);
        assert_eq!(detect_mime_type("test.txt", None).unwrap(), mime::TEXT_PLAIN);
        assert_eq!(detect_mime_type("test.json", None).unwrap(), mime::APPLICATION_JSON);
    }

    #[test]
    fn test_file_validation() {
        let data = b"Hello, world!";
        let file = File::from_bytes("test.txt", Bytes::from_static(data), None).unwrap();
        
        let constraints = FileConstraints {
            max_size: 10,
            allowed_types: None,
            require_hash: false,
        };
        
        // Should fail size validation
        assert!(file.validate(&constraints).is_err());
    }

    #[test]
    fn test_file_type_checks() {
        let image_file = File::from_bytes("test.jpg", Bytes::new(), Some(mime::IMAGE_JPEG)).unwrap();
        let text_file = File::from_bytes("test.txt", Bytes::new(), Some(mime::TEXT_PLAIN)).unwrap();
        
        assert!(image_file.is_image());
        assert!(!image_file.is_text());
        
        assert!(text_file.is_text());
        assert!(!text_file.is_image());
    }
}

