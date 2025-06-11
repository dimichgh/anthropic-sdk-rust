use anthropic_sdk::{
    Anthropic, MessageCreateBuilder, ContentBlockParam, MessageContent,
    File, FileSource, FileBuilder, FileConstraints, to_file, FileError,
};
use bytes::Bytes;
use mime::Mime;
use std::path::Path;
use tokio::fs;
use base64::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("🚀 Anthropic SDK - File Upload System Demo");
    println!("==========================================\n");

    // Demo 1: Create files from different sources
    demo_file_creation().await?;
    
    // Demo 2: File validation and constraints
    demo_file_validation().await?;
    
    // Demo 3: MIME type detection
    demo_mime_detection().await?;
    
    // Demo 4: File processing utilities
    demo_file_processing().await?;
    
    // Demo 5: Integration with messages (mock - requires API key)
    demo_message_integration().await?;

    println!("\n✅ All file upload demos completed successfully!");
    Ok(())
}

async fn demo_file_creation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Demo 1: Creating Files from Different Sources");
    println!("-------------------------------------------------");

    // 1. From bytes
    let text_data = b"Hello, world! This is a text file.";
    let text_file = File::from_bytes("hello.txt", Bytes::from_static(text_data), None)?;
    println!("✓ Created from bytes: {}", text_file);

    // 2. From base64
    let base64_data = base64::engine::general_purpose::STANDARD.encode(text_data);
    let base64_file = File::from_base64("hello_b64.txt", base64_data, None)?;
    println!("✓ Created from base64: {}", base64_file);

    // 3. Using FileBuilder with constraints
    let constrained_file = FileBuilder::new()
        .name("constrained.txt")
        .mime_type(mime::TEXT_PLAIN)
        .constraints(FileConstraints {
            max_size: 1024 * 1024, // 1MB
            allowed_types: Some(vec![mime::TEXT_PLAIN]),
            require_hash: false,
        })
        .with_hash()
        .build(FileSource::Bytes(Bytes::from_static(text_data)))
        .await?;
    println!("✓ Created with builder: {}", constrained_file);
    println!("  Hash: {:?}", constrained_file.hash);

    // 4. Using convenience function
    let convenient_file = to_file(
        FileSource::Bytes(Bytes::from_static(b"Convenient creation")),
        Some("convenient.txt".to_string()),
        Some(mime::TEXT_PLAIN),
    ).await?;
    println!("✓ Created with to_file(): {}", convenient_file);

    println!();
    Ok(())
}

async fn demo_file_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Demo 2: File Validation and Constraints");
    println!("------------------------------------------");

    let large_data = vec![0u8; 1024 * 1024]; // 1MB of zeros
    let large_file = File::from_bytes("large.bin", Bytes::from(large_data), None)?;

    // Test size constraints
    let strict_constraints = FileConstraints {
        max_size: 1024, // 1KB limit
        allowed_types: None,
        require_hash: false,
    };

    match large_file.validate(&strict_constraints) {
        Err(FileError::TooLarge { size, max_size }) => {
            println!("✓ Size validation works: {} bytes > {} bytes limit", size, max_size);
        }
        _ => println!("❌ Size validation failed"),
    }

    // Test MIME type constraints
    let image_constraints = FileConstraints {
        max_size: 10 * 1024 * 1024, // 10MB
        allowed_types: Some(vec![mime::IMAGE_JPEG, mime::IMAGE_PNG]),
        require_hash: false,
    };

    let text_file = File::from_bytes("text.txt", Bytes::from_static(b"text"), None)?;
    match text_file.validate(&image_constraints) {
        Err(FileError::InvalidMimeType { mime_type, .. }) => {
            println!("✓ MIME type validation works: {} not allowed", mime_type);
        }
        _ => println!("❌ MIME type validation failed"),
    }

    println!();
    Ok(())
}

async fn demo_mime_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Demo 3: MIME Type Detection");
    println!("------------------------------");

    // Test extension-based detection
    let files = vec![
        ("document.pdf", "dummy pdf content".as_bytes()),
        ("image.jpg", "dummy jpeg content here".as_bytes()),
        ("data.json", r#"{"key": "value"}"#.as_bytes()),
        ("archive.zip", "dummy zip content".as_bytes()),
        ("video.mp4", "dummy mp4 content".as_bytes()),
    ];

    for (name, content) in files {
        let file = File::from_bytes(name, Bytes::from(content), None)?;
        println!("✓ {}: {}", name, file.mime_type);
    }

    // Test magic bytes detection (PNG example)
    let png_magic = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG signature
    let png_file = File::from_bytes("image_no_ext", Bytes::from(png_magic.to_vec()), None)?;
    println!("✓ Magic bytes detection: {}", png_file.mime_type);

    println!();
    Ok(())
}

async fn demo_file_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️ Demo 4: File Processing Utilities");
    println!("-----------------------------------");

    let text_data = b"Hello, world! This is sample content for processing.";
    let mut file = File::from_bytes("sample.txt", Bytes::from_static(text_data), None)?;

    // Calculate hash
    let hash = file.calculate_hash().await?;
    println!("✓ File hash calculated: {}", hash);

    // Verify hash
    let is_valid = file.verify_hash(&hash).await?;
    println!("✓ Hash verification: {}", is_valid);

    // Convert to base64
    let base64_data = file.to_base64().await?;
    println!("✓ Base64 conversion: {}...", &base64_data[..20]);

    // Convert back to bytes
    let bytes = file.to_bytes().await?;
    println!("✓ Bytes conversion: {} bytes", bytes.len());

    // File type checks
    println!("✓ Type checks:");
    println!("  - Is image: {}", file.is_image());
    println!("  - Is text: {}", file.is_text());
    println!("  - Is application: {}", file.is_application());

    // Create an image file example
    let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic bytes
    let image_file = File::from_bytes("test.jpg", Bytes::from(jpeg_data), None)?;
    println!("✓ Image file type check: {}", image_file.is_image());

    println!();
    Ok(())
}

async fn demo_message_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("💬 Demo 5: Integration with Messages");
    println!("------------------------------------");

    // Create a sample image file (mock JPEG)
    let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]; // JPEG header
    let image_file = File::from_bytes("chart.jpg", Bytes::from(jpeg_data), Some(mime::IMAGE_JPEG))?;
    
    // Create content blocks with file
    let image_block = ContentBlockParam::image_file(image_file).await?;
    let text_block = ContentBlockParam::text("Please analyze this chart");

    // Build message content
    let message_content = MessageContent::Blocks(vec![text_block, image_block]);

    // Create message builder (this would normally use an API key)
    let message_params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
        .user(message_content)
        .system("You are a helpful data analyst")
        .temperature(0.3)
        .build();

    println!("✓ Message created with file attachment");
    println!("  Model: {}", message_params.model);
    println!("  Messages: {}", message_params.messages.len());
    println!("  System prompt: {:?}", message_params.system);

    // Display content structure
    if let MessageContent::Blocks(blocks) = &message_params.messages[0].content {
        println!("  Content blocks: {}", blocks.len());
        for (i, block) in blocks.iter().enumerate() {
            match block {
                ContentBlockParam::Text { .. } => println!("    Block {}: Text", i),
                ContentBlockParam::Image { .. } => println!("    Block {}: Image", i),
                _ => println!("    Block {}: Other", i),
            }
        }
    }

    println!("✓ File integration with messages working correctly");

    // Demo: Multiple file types in one message
    let text_file = File::from_bytes("document.txt", Bytes::from_static(b"Document content"), None)?;
    let pdf_data = vec![0x25, 0x50, 0x44, 0x46]; // PDF magic bytes
    let pdf_file = File::from_bytes("report.pdf", Bytes::from(pdf_data), Some("application/pdf".parse()?))?;

    println!("✓ Created multiple file types:");
    println!("  - Text file: {} ({})", text_file.name, text_file.mime_type);
    println!("  - PDF file: {} ({})", pdf_file.name, pdf_file.mime_type);

    println!();
    Ok(())
}

// Helper function to demonstrate file creation patterns
#[allow(dead_code)]
async fn create_sample_files() -> Result<Vec<File>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    // Text file
    files.push(File::from_bytes(
        "readme.txt",
        Bytes::from_static(b"This is a README file"),
        Some(mime::TEXT_PLAIN),
    )?);

    // JSON file
    files.push(File::from_bytes(
        "config.json",
        Bytes::from_static(b"{\"version\": \"1.0\", \"debug\": true}"),
        Some(mime::APPLICATION_JSON),
    )?);

    // Mock image file
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG signature
    files.push(File::from_bytes(
        "logo.png",
        Bytes::from(png_data),
        Some(mime::IMAGE_PNG),
    )?);

    Ok(files)
}

// Helper function to show advanced file operations
#[allow(dead_code)]
async fn advanced_file_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Advanced File Operations");
    println!("---------------------------");

    // Create a large file for testing
    let large_content = "x".repeat(1024 * 100); // 100KB
    let mut large_file = File::from_bytes("large.txt", Bytes::from(large_content.into_bytes()), None)?;

    // Calculate hash with timing
    let start = std::time::Instant::now();
    let hash = large_file.calculate_hash().await?;
    let duration = start.elapsed();
    
    println!("✓ Hash calculation for 100KB file: {:?}", duration);
    println!("  Hash: {}", hash);

    // File size validation
    let constraints = FileConstraints {
        max_size: 1024 * 50, // 50KB limit
        allowed_types: None,
        require_hash: false,
    };

    match large_file.validate(&constraints) {
        Err(FileError::TooLarge { size, max_size }) => {
            println!("✓ Size constraint enforced: {} > {}", size, max_size);
        }
        Ok(_) => println!("❌ Size constraint not enforced"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }

    Ok(())
} 