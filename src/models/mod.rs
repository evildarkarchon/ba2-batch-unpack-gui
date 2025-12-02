//! Data models for the UI
//!
//! This module provides data structures used by the UI layer, including:
//! - File entries for the preview table
//! - Sorting and comparison logic
//! - Display formatting helpers

use crate::operations::{format_size, BA2FileInfo};
use std::cmp::Ordering;
use std::path::PathBuf;

/// File entry for display in the preview table
///
/// This struct represents a BA2 file discovered during scanning,
/// with additional functionality for sorting and display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// File name (without path)
    pub file_name: String,

    /// File size in bytes
    pub file_size: u64,

    /// Number of files contained in the archive
    pub num_files: u32,

    /// Parent directory name (mod folder)
    pub dir_name: String,

    /// Full path to the file
    pub full_path: PathBuf,

    /// Whether the file appears to be corrupted
    pub is_bad: bool,
}

impl FileEntry {
    /// Create a new `FileEntry`
    pub const fn new(
        file_name: String,
        file_size: u64,
        num_files: u32,
        dir_name: String,
        full_path: PathBuf,
        is_bad: bool,
    ) -> Self {
        Self {
            file_name,
            file_size,
            num_files,
            dir_name,
            full_path,
            is_bad,
        }
    }

    /// Get human-readable file size (e.g., "10.5 MiB")
    pub fn size_display(&self) -> String {
        format_size(self.file_size)
    }

    /// Get file name for display
    pub fn name_display(&self) -> &str {
        &self.file_name
    }

    /// Get number of files for display
    pub fn file_count_display(&self) -> String {
        self.num_files.to_string()
    }

    /// Get mod folder name for display
    pub fn mod_display(&self) -> &str {
        &self.dir_name
    }

    /// Check if this file is marked as bad
    pub const fn is_corrupted(&self) -> bool {
        self.is_bad
    }
}

/// Convert from `BA2FileInfo` to `FileEntry`
impl From<BA2FileInfo> for FileEntry {
    fn from(info: BA2FileInfo) -> Self {
        Self {
            file_name: info.file_name,
            file_size: info.file_size,
            num_files: info.num_files,
            dir_name: info.dir_name,
            full_path: info.full_path,
            is_bad: info.is_bad,
        }
    }
}

/// Sorting criteria for file entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    /// Sort by file name (alphabetically)
    Name,
    /// Sort by file size (largest first)
    Size,
    /// Sort by number of files (most first)
    FileCount,
    /// Sort by mod folder name (alphabetically)
    ModName,
}

impl FileEntry {
    /// Compare two entries based on a sorting criterion
    pub fn compare(&self, other: &Self, sort_by: SortBy) -> Ordering {
        match sort_by {
            SortBy::Name => self.file_name.cmp(&other.file_name),
            SortBy::Size => self.file_size.cmp(&other.file_size), // Smallest first (Natural)
            SortBy::FileCount => self.num_files.cmp(&other.num_files), // Fewest first (Natural)
            SortBy::ModName => self.dir_name.cmp(&other.dir_name),
        }
    }
}

/// Default ordering: by file size (largest first)
impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Default to size descending for Ord implementation (legacy/default behavior)
        other.file_size.cmp(&self.file_size)
    }
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Collection of file entries for the preview table
#[derive(Debug, Clone, Default)]
pub struct FileEntryList {
    entries: Vec<FileEntry>,
}

impl FileEntryList {
    /// Create a new empty list
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Create from a vector of entries
    pub const fn from_vec(entries: Vec<FileEntry>) -> Self {
        Self { entries }
    }

    /// Create from `BA2FileInfo` results
    pub fn from_scan_results(results: Vec<BA2FileInfo>) -> Self {
        Self {
            entries: results.into_iter().map(FileEntry::from).collect(),
        }
    }

    /// Add an entry to the list
    pub fn push(&mut self, entry: FileEntry) {
        self.entries.push(entry);
    }

    /// Get the number of entries
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the list is empty
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get a reference to all entries
    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    /// Get a mutable reference to all entries
    pub const fn entries_mut(&mut self) -> &mut Vec<FileEntry> {
        &mut self.entries
    }

    /// Sort entries by a specific criterion
    pub fn sort_by(&mut self, sort_by: SortBy, reverse: bool) {
        self.entries.sort_by(|a, b| {
            let ord = a.compare(b, sort_by);
            if reverse {
                ord.reverse()
            } else {
                ord
            }
        });
    }

    /// Get entry at index
    pub fn get(&self, index: usize) -> Option<&FileEntry> {
        self.entries.get(index)
    }

    /// Remove entry at index
    pub fn remove(&mut self, index: usize) -> Option<FileEntry> {
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }

    /// Get total size of all files
    pub fn total_size(&self) -> u64 {
        self.entries.iter().map(|e| e.file_size).sum()
    }

    /// Get total number of files across all archives
    pub fn total_file_count(&self) -> u32 {
        self.entries.iter().map(|e| e.num_files).sum()
    }

    /// Get count of corrupted files
    pub fn bad_file_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_bad).count()
    }

    /// Filter entries to remove corrupted files
    pub fn filter_bad_files(&mut self) {
        self.entries.retain(|e| !e.is_bad);
    }

    /// Get indices of bad files
    pub fn bad_file_indices(&self) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| if entry.is_bad { Some(idx) } else { None })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(name: &str, size: u64, num_files: u32, is_bad: bool) -> FileEntry {
        FileEntry::new(
            name.to_string(),
            size,
            num_files,
            "TestMod".to_string(),
            PathBuf::from(format!("/path/to/{}", name)),
            is_bad,
        )
    }

    #[test]
    fn test_file_entry_creation() {
        let entry = create_test_entry("test.ba2", 1000, 10, false);
        assert_eq!(entry.file_name, "test.ba2");
        assert_eq!(entry.file_size, 1000);
        assert_eq!(entry.num_files, 10);
        assert!(!entry.is_bad);
    }

    #[test]
    fn test_file_entry_display_methods() {
        let entry = create_test_entry("test.ba2", 1500, 25, false);
        assert_eq!(entry.name_display(), "test.ba2");
        assert_eq!(entry.file_count_display(), "25");
        assert_eq!(entry.mod_display(), "TestMod");
        assert!(!entry.size_display().is_empty());
    }

    #[test]
    fn test_sorting_by_name() {
        let mut entries = vec![
            create_test_entry("zebra.ba2", 1000, 10, false),
            create_test_entry("alpha.ba2", 2000, 20, false),
            create_test_entry("beta.ba2", 1500, 15, false),
        ];

        // Ascending
        entries.sort_by(|a, b| a.compare(b, SortBy::Name));
        assert_eq!(entries[0].file_name, "alpha.ba2");
        assert_eq!(entries[1].file_name, "beta.ba2");
        assert_eq!(entries[2].file_name, "zebra.ba2");
    }

    #[test]
    fn test_sorting_by_size() {
        let mut entries = vec![
            create_test_entry("small.ba2", 1000, 10, false),
            create_test_entry("large.ba2", 3000, 30, false),
            create_test_entry("medium.ba2", 2000, 20, false),
        ];

        // Ascending (Natural)
        entries.sort_by(|a, b| a.compare(b, SortBy::Size));
        assert_eq!(entries[0].file_name, "small.ba2");
        assert_eq!(entries[1].file_name, "medium.ba2");
        assert_eq!(entries[2].file_name, "large.ba2");
    }

    #[test]
    fn test_sorting_by_file_count() {
        let mut entries = vec![
            create_test_entry("few.ba2", 1000, 5, false),
            create_test_entry("many.ba2", 1000, 50, false),
            create_test_entry("some.ba2", 1000, 20, false),
        ];

        // Ascending (Natural)
        entries.sort_by(|a, b| a.compare(b, SortBy::FileCount));
        assert_eq!(entries[0].file_name, "few.ba2");
        assert_eq!(entries[1].file_name, "some.ba2");
        assert_eq!(entries[2].file_name, "many.ba2");
    }

    #[test]
    fn test_default_ordering() {
        let small = create_test_entry("small.ba2", 1000, 10, false);
        let large = create_test_entry("large.ba2", 2000, 20, false);

        // Ord implementation is still Descending Size
        assert_eq!(small.cmp(&large), Ordering::Greater); 
        assert_eq!(large.cmp(&small), Ordering::Less);
    }

    #[test]
    fn test_file_entry_list_sorting() {
        let mut list = FileEntryList::from_vec(vec![
            create_test_entry("zebra.ba2", 1000, 10, false),
            create_test_entry("alpha.ba2", 2000, 20, false),
        ]);

        // Ascending
        list.sort_by(SortBy::Name, false);
        assert_eq!(list.entries()[0].file_name, "alpha.ba2");
        assert_eq!(list.entries()[1].file_name, "zebra.ba2");

        // Descending
        list.sort_by(SortBy::Name, true);
        assert_eq!(list.entries()[0].file_name, "zebra.ba2");
        assert_eq!(list.entries()[1].file_name, "alpha.ba2");
    }

    #[test]
    fn test_bad_file_handling() {
        let list = FileEntryList::from_vec(vec![
            create_test_entry("good1.ba2", 1000, 10, false),
            create_test_entry("bad.ba2", 2000, 0, true),
            create_test_entry("good2.ba2", 3000, 30, false),
        ]);

        assert_eq!(list.bad_file_count(), 1);

        let bad_indices = list.bad_file_indices();
        assert_eq!(bad_indices, vec![1]);
    }

    #[test]
    fn test_filter_bad_files() {
        let mut list = FileEntryList::from_vec(vec![
            create_test_entry("good1.ba2", 1000, 10, false),
            create_test_entry("bad.ba2", 2000, 0, true),
            create_test_entry("good2.ba2", 3000, 30, false),
        ]);

        list.filter_bad_files();
        assert_eq!(list.len(), 2);
        assert_eq!(list.bad_file_count(), 0);
    }

    #[test]
    fn test_from_ba2fileinfo() {
        let ba2_info = BA2FileInfo {
            file_name: "test.ba2".to_string(),
            file_size: 1000,
            num_files: 10,
            dir_name: "TestMod".to_string(),
            full_path: PathBuf::from("/path/to/test.ba2"),
            is_bad: false,
        };

        let entry: FileEntry = ba2_info.into();
        assert_eq!(entry.file_name, "test.ba2");
        assert_eq!(entry.file_size, 1000);
    }
}
