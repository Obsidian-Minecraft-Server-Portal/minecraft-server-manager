use log::{debug, error, info, warn};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSystemEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub r#type: String,
    pub mime: Option<String>,
    pub category: FileMimeCategory,
    pub created: SystemTime,
    pub last_modified: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSystemEntries {
    pub parent: Option<PathBuf>,
    pub entries: Vec<FileSystemEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileMimeCategory {
    TEXT,
    IMAGE,
    AUDIO,
    ARCHIVE,
    VIDEO,
    UNKNOWN,
}
fn get_file_type(extension: String) -> String {
    let types: HashMap<&str, &str> = HashMap::from([
        ("zip", "Zip Archive"),
        ("tar", "Tar Archive"),
        ("tar.gz", "Tar GZip Archive"),
        ("tar.bz2", "Tar BZip2 Archive"),
        ("tar.xz", "Tar XZ Archive"),
        ("7z", "7-Zip Archive"),
        ("rar", "RAR Archive"),
        ("jar", "Java Archive"),
        ("war", "Web Archive"),
        ("ear", "Enterprise Archive"),
        ("exe", "Windows Executable"),
        ("msi", "Windows Installer"),
        ("sh", "Shell Script"),
        ("bat", "Batch Script"),
        ("cmd", "Command Script"),
        ("py", "Python Script"),
        ("rb", "Ruby Script"),
        ("pl", "Perl Script"),
        ("php", "PHP Script"),
        ("html", "HTML Document"),
        ("htm", "HTML Document"),
        ("xhtml", "XHTML Document"),
        ("css", "CSS Stylesheet"),
        ("js", "JavaScript File"),
        ("ts", "TypeScript File"),
        ("jsx", "JavaScript XML"),
        ("tsx", "TypeScript XML"),
        ("json", "JSON File"),
        ("xml", "XML Document"),
        ("yaml", "YAML Document"),
        ("yml", "YAML Document"),
        ("toml", "TOML Config"),
        ("ini", "INI Config"),
        ("cfg", "Configuration File"),
        ("conf", "Configuration File"),
        ("log", "Log File"),
        ("md", "Markdown Document"),
        ("txt", "Text File"),
        ("csv", "CSV File"),
        ("tsv", "TSV File"),
        ("pdf", "PDF Document"),
        ("doc", "Word Document"),
        ("docx", "Word Document"),
        ("xls", "Excel Spreadsheet"),
        ("xlsx", "Excel Spreadsheet"),
        ("ppt", "PowerPoint Presentation"),
        ("pptx", "PowerPoint Presentation"),
        ("odt", "OpenDocument Text"),
        ("ods", "OpenDocument Spreadsheet"),
        ("odp", "OpenDocument Presentation"),
        ("jpg", "JPEG Image"),
        ("jpeg", "JPEG Image"),
        ("png", "PNG Image"),
        ("gif", "GIF Image"),
        ("bmp", "Bitmap Image"),
        ("tiff", "TIFF Image"),
        ("ico", "Icon Image"),
        ("svg", "SVG Image"),
        ("mp3", "MP3 Audio"),
        ("wav", "WAV Audio"),
        ("flac", "FLAC Audio"),
        ("ogg", "OGG Audio"),
        ("aac", "AAC Audio"),
        ("m4a", "M4A Audio"),
        ("wma", "WMA Audio"),
        ("mp4", "MP4 Video"),
        ("m4v", "M4V Video"),
        ("mkv", "MKV Video"),
        ("avi", "AVI Video"),
        ("mov", "MOV Video"),
        ("wmv", "WMV Video"),
        ("flv", "FLV Video"),
        ("webm", "WebM Video"),
        ("vob", "DVD Video"),
        ("mpg", "MPEG Video"),
        ("mpeg", "MPEG Video"),
        ("iso", "ISO Disk Image"),
        ("dmg", "MacOS Disk Image"),
        ("vdi", "VirtualBox Disk Image"),
        ("vmdk", "VMware Disk Image"),
        ("qcow2", "QEMU Copy-On-Write Disk Image"),
        ("qcow", "QEMU Copy-On-Write Disk Image"),
        ("ova", "Virtual Appliance"),
        ("ovf", "Open Virtualization Format"),
        ("img", "Disk Image"),
        ("dd", "Disk Dump Image"),
        ("vhd", "Virtual Hard Disk"),
        ("vhdx", "Virtual Hard Disk"),
        ("xpi", "Mozilla Add-on"),
        ("crx", "Chrome Extension"),
        ("oxt", "OpenOffice Extension"),
        ("apk", "Android Package"),
        ("ipa", "iOS App Package"),
        ("deb", "Debian Package"),
        ("rpm", "Red Hat Package"),
        ("flatpak", "Flatpak Package"),
        ("mcworld", "Minecraft World"),
        ("mcpack", "Minecraft Resource Pack"),
        ("mcaddon", "Minecraft Add-On"),
        ("mctemplate", "Minecraft Template"),
        ("mclevel", "Minecraft Level"),
        ("schematic", "Minecraft Schematic"),
        ("dat", "Minecraft Data File"),
        ("ldb", "Minecraft LevelDB Database File"),
        ("mca", "Minecraft Anvil Data"),
        ("mcr", "Minecraft Region Data"),
        ("nbt", "Minecraft Named Binary Tag"),
        ("mcfunction", "Minecraft Function File"),
        ("mcmeta", "Minecraft Metadata File"),
        ("properties", "Minecraft Properties File"),
    ]);

    types
        .get(&extension[..])
        .unwrap_or(&extension.as_str())
        .to_string()
}

fn get_mime_category(path: impl AsRef<Path>) -> FileMimeCategory {
    let path_ref = path.as_ref();

    // Debug: Check input path
    debug!("Determining MIME category for path: {:?}", path_ref);

    // Check existence and if it's a directory
    if !path_ref.exists() {
        warn!("Path does not exist: {:?}", path_ref);
        return FileMimeCategory::UNKNOWN;
    } else if path_ref.is_dir() {
        info!("Path is a directory, not a file: {:?}", path_ref);
        return FileMimeCategory::UNKNOWN;
    }

    let mime = mime_guess::from_path(&path).first();

    if let Some(mime) = mime {
        let mime_type = mime.type_().as_str();
        debug!(
            "MIME type identified: {:?} for path: {:?}",
            mime_type, path_ref
        );

        match mime_type {
            "text" => FileMimeCategory::TEXT,
            "image" => FileMimeCategory::IMAGE,
            "audio" => FileMimeCategory::AUDIO,
            "video" => FileMimeCategory::VIDEO,
            "application" => FileMimeCategory::ARCHIVE,
            _ => {
                warn!(
                    "Unknown MIME type: {:?} for path: {:?}",
                    mime_type, path_ref
                );
                FileMimeCategory::UNKNOWN
            }
        }
    } else {
        warn!("No MIME type could be identified for path: {:?}", path_ref);

        if is_text_file(path) {
            info!(
                "Path: {:?} identified as a text file based on content analysis.",
                path_ref
            );
            FileMimeCategory::TEXT
        } else {
            warn!(
                "Content analysis indicates unknown MIME category for path: {:?}",
                path_ref
            );
            FileMimeCategory::UNKNOWN
        }
    }
}

fn is_text_file(file_path: impl AsRef<Path>) -> bool {
    let path = file_path.as_ref();
    const BUFFER_SIZE: usize = 1024;

    debug!("Checking if path is a text file: {:?}", path);

    match File::open(path) {
        Ok(mut file) => {
            let mut buffer = [0; BUFFER_SIZE];
            match file.read(&mut buffer) {
                Ok(bytes_read) => {
                    debug!("Read {} bytes from file: {:?}", bytes_read, path);
                    for &byte in &buffer[..bytes_read] {
                        if !(byte == 0x09
                            || byte == 0x0A
                            || byte == 0x0D
                            || (0x20..=0x7E).contains(&byte))
                        {
                            debug!(
                                "Non-text byte identified in file: {:?}. It is not a text file.",
                                path
                            );
                            return false;
                        }
                    }
                    debug!("File appears to be a text file: {:?}", path);
                    true
                }
                Err(err) => {
                    error!("Failed to read file: {:?}. Error: {:?}", path, err);
                    false
                }
            }
        }
        Err(err) => {
            error!("Failed to open file: {:?}. Error: {:?}", path, err);
            false
        }
    }
}

fn get_mime(path: impl AsRef<Path>) -> Option<String> {
    let path_ref = path.as_ref();
    debug!("Getting MIME type for path: {:?}", path_ref);

    mime_guess::from_path(path).first().map(|m| {
        let mime = m.to_string();
        debug!("MIME type for path {:?}: {:?}", path_ref, mime);
        mime
    })
}

impl Default for FileSystemEntry {
    fn default() -> Self {
        info!("Creating default FileSystemEntry.");
        Self {
            name: "".to_string(),
            path: Default::default(),
            is_dir: false,
            size: 0,
            r#type: "".to_string(),
            mime: None,
            category: FileMimeCategory::TEXT,
            created: SystemTime::now(),
            last_modified: SystemTime::now(),
        }
    }
}

impl Default for FileSystemEntries {
    fn default() -> Self {
        info!("Creating default FileSystemEntries.");
        Self {
            parent: None,
            entries: Vec::new(),
        }
    }
}

impl From<PathBuf> for FileSystemEntry {
    fn from(value: PathBuf) -> Self {
        debug!(
            "Converting PathBuf to FileSystemEntry for path: {:?}",
            value
        );
        match value.metadata() {
            Ok(metadata) => {
                debug!("Metadata retrieved for path: {:?}", value);

                Self {
                    name: value
                        .file_name()
                        .unwrap_or(OsStr::new(""))
                        .to_string_lossy()
                        .to_string(),

                    path: value.clone(),
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                    r#type: get_file_type(
                        value
                            .extension()
                            .unwrap_or(OsStr::new(""))
                            .to_string_lossy()
                            .to_string(),
                    ),
                    mime: get_mime(&value),
                    category: get_mime_category(&value),
                    created: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
                    last_modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                }
            }
            Err(err) => {
                error!(
                    "Failed to retrieve metadata for path: {:?}. Error: {:?}",
                    value, err
                );
                Self::default()
            }
        }
    }
}

impl From<PathBuf> for FileSystemEntries {
    fn from(value: PathBuf) -> Self {
        debug!(
            "Converting PathBuf to FileSystemEntries for directory: {:?}",
            value
        );
        match fs::read_dir(&value) {
            Ok(directory_entries) => {
                let mut entries: Vec<FileSystemEntry> = Vec::new();
                for entry in directory_entries.flatten() {
                    debug!("Processing directory entry: {:?}", entry.path());
                    entries.push(entry.path().into());
                }
                info!("Directory processed successfully: {:?}", value);

                Self {
                    parent: value.parent().map(|p| p.to_path_buf()),
                    entries,
                }
            }
            Err(err) => {
                error!("Failed to read directory: {:?}. Error: {:?}", value, err);
                Self::default()
            }
        }
    }
}
