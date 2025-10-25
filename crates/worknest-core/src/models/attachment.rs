//! Attachment domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{TicketId, UserId};
use crate::error::{CoreError, Result};

/// Unique identifier for attachments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttachmentId(Uuid);

impl AttachmentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Ok(Self(
            Uuid::parse_str(s).map_err(|e| CoreError::InvalidId(e.to_string()))?,
        ))
    }
}

impl Default for AttachmentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AttachmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// File attachment for a ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub ticket_id: TicketId,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub file_path: String,
    pub uploaded_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl Attachment {
    /// Create a new attachment
    pub fn new(
        ticket_id: TicketId,
        filename: String,
        file_size: i64,
        mime_type: String,
        file_path: String,
        uploaded_by: UserId,
    ) -> Self {
        Self {
            id: AttachmentId::new(),
            ticket_id,
            filename,
            file_size,
            mime_type,
            file_path,
            uploaded_by,
            created_at: Utc::now(),
        }
    }

    /// Validate the attachment
    pub fn validate(&self) -> Result<()> {
        if self.filename.trim().is_empty() {
            return Err(CoreError::Validation(
                "Filename cannot be empty".to_string(),
            ));
        }

        if self.file_size <= 0 {
            return Err(CoreError::Validation(
                "File size must be positive".to_string(),
            ));
        }

        // 100MB max file size
        if self.file_size > 100 * 1024 * 1024 {
            return Err(CoreError::Validation(
                "File size cannot exceed 100MB".to_string(),
            ));
        }

        if self.mime_type.is_empty() {
            return Err(CoreError::Validation(
                "MIME type cannot be empty".to_string(),
            ));
        }

        if self.file_path.is_empty() {
            return Err(CoreError::Validation(
                "File path cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.filename.rsplit('.').next()
    }

    /// Check if file is an image
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    /// Format file size as human-readable string
    pub fn formatted_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = self.file_size as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_attachment() {
        let attachment = Attachment::new(
            TicketId::new(),
            "test.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            "/path/to/test.pdf".to_string(),
            UserId::new(),
        );

        assert_eq!(attachment.filename, "test.pdf");
        assert_eq!(attachment.file_size, 1024);
        assert!(attachment.validate().is_ok());
    }

    #[test]
    fn test_empty_filename_validation() {
        let attachment = Attachment::new(
            TicketId::new(),
            "   ".to_string(),
            1024,
            "application/pdf".to_string(),
            "/path/to/file".to_string(),
            UserId::new(),
        );
        assert!(attachment.validate().is_err());
    }

    #[test]
    fn test_file_size_validation() {
        let attachment = Attachment::new(
            TicketId::new(),
            "test.pdf".to_string(),
            0,
            "application/pdf".to_string(),
            "/path/to/file".to_string(),
            UserId::new(),
        );
        assert!(attachment.validate().is_err());

        let large_attachment = Attachment::new(
            TicketId::new(),
            "test.pdf".to_string(),
            101 * 1024 * 1024,
            "application/pdf".to_string(),
            "/path/to/file".to_string(),
            UserId::new(),
        );
        assert!(large_attachment.validate().is_err());
    }

    #[test]
    fn test_extension() {
        let attachment = Attachment::new(
            TicketId::new(),
            "document.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            "/path/to/file".to_string(),
            UserId::new(),
        );
        assert_eq!(attachment.extension(), Some("pdf"));
    }

    #[test]
    fn test_is_image() {
        let image = Attachment::new(
            TicketId::new(),
            "photo.jpg".to_string(),
            1024,
            "image/jpeg".to_string(),
            "/path/to/file".to_string(),
            UserId::new(),
        );
        assert!(image.is_image());

        let pdf = Attachment::new(
            TicketId::new(),
            "doc.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            "/path/to/file".to_string(),
            UserId::new(),
        );
        assert!(!pdf.is_image());
    }

    #[test]
    fn test_formatted_size() {
        let attachment = Attachment::new(
            TicketId::new(),
            "test.pdf".to_string(),
            1536,
            "application/pdf".to_string(),
            "/path/to/file".to_string(),
            UserId::new(),
        );
        assert_eq!(attachment.formatted_size(), "1.50 KB");
    }
}
