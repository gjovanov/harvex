use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Pdf,
    Image,
    Excel,
    Word,
    Unknown(String),
}

impl FileType {
    /// Detect file type from filename extension and content type.
    pub fn detect(filename: &str, content_type: &str) -> Self {
        // Try content type first
        match content_type {
            "application/pdf" => return Self::Pdf,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-excel"
            | "text/csv" => return Self::Excel,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/msword" => return Self::Word,
            ct if ct.starts_with("image/") => return Self::Image,
            _ => {}
        }

        // Fall back to extension
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "pdf" => Self::Pdf,
            "png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp" | "webp" | "gif" => Self::Image,
            "xlsx" | "xls" | "csv" | "ods" => Self::Excel,
            "docx" | "doc" => Self::Word,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Pdf => "PDF",
            Self::Image => "Image",
            Self::Excel => "Excel",
            Self::Word => "Word",
            Self::Unknown(_) => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_by_content_type() {
        assert_eq!(FileType::detect("file.bin", "application/pdf"), FileType::Pdf);
        assert_eq!(FileType::detect("file.bin", "image/png"), FileType::Image);
        assert_eq!(
            FileType::detect(
                "file.bin",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            ),
            FileType::Excel
        );
    }

    #[test]
    fn detect_by_extension() {
        assert_eq!(
            FileType::detect("invoice.pdf", "application/octet-stream"),
            FileType::Pdf
        );
        assert_eq!(
            FileType::detect("photo.jpg", "application/octet-stream"),
            FileType::Image
        );
        assert_eq!(
            FileType::detect("data.xlsx", "application/octet-stream"),
            FileType::Excel
        );
        assert_eq!(
            FileType::detect("report.docx", "application/octet-stream"),
            FileType::Word
        );
    }
}
