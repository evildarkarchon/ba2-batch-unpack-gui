# Manual UI Testing Checklist for Unpackrr-rs

This checklist covers manual testing scenarios for the Slint-based UI that cannot be easily automated.

## Pre-Testing Setup
- [ ] Build the application: `cargo build --release`
- [ ] Prepare test BA2 files (various sizes and types)
- [ ] Clear any existing configuration to test first-run experience

## Extraction Screen (Main Workflow)

### Folder Selection
- [ ] Click "Browse" button opens native folder picker
- [ ] Selected folder path appears in text field
- [ ] Selected folder is saved to config and persists on restart

### BA2 Scanning
- [ ] Click "Scan" button starts scanning
- [ ] Status text shows "Scanning..." during operation
- [ ] File table populates with discovered BA2 files
- [ ] Total files count updates correctly
- [ ] Total size displays in human-readable format (KB, MB, GB)
- [ ] Corrupted BA2 files highlighted in dark red
- [ ] Empty table shows helpful empty state message

### File Table
- [ ] Columns display correctly: Filename, Size, File Count, Mod Folder
- [ ] Click column headers to sort (Name, Size, FileCount, ModName)
- [ ] Sort indicator shows current sort column
- [ ] Row selection highlights selected row
- [ ] Three-dots menu (⋮) appears on hover
- [ ] Click ⋮ opens context menu with "Ignore" and "Open" options
- [ ] "Ignore" removes file from list
- [ ] "Open" launches external tool (if configured)

### Size Threshold Filtering
- [ ] "Auto" toggle calculates threshold based on 235-file limit
- [ ] Manual input accepts numeric values
- [ ] Table filters to show only files above threshold
- [ ] Total files/size updates to reflect filtered view

### Extraction Process
- [ ] Click "Start Extraction" begins extraction
- [ ] Progress bar animates smoothly
- [ ] Current file name displays during extraction
- [ ] File index shows (e.g., "3/10")
- [ ] Extraction speed displays (files/s or s/file)
- [ ] ETA displays in h/m/s format
- [ ] Pause button pauses extraction
- [ ] Resume button resumes from pause
- [ ] Cancel button stops extraction
- [ ] Success/failure counts displayed at completion
- [ ] "Open Folder" button appears after completion
- [ ] "Open Folder" opens extraction directory

## Check Files Screen (Validation)

### Folder Selection
- [ ] Browse button opens folder picker
- [ ] Selected path displays correctly

### Validation Options
- [ ] "Deep Scan" checkbox toggles
- [ ] Quick scan lists BA2 contents
- [ ] Deep scan extracts to temp and verifies

### Validation Results
- [ ] Results display in scrollable text area
- [ ] Corrupted files clearly indicated
- [ ] File count statistics shown
- [ ] Long results scrollable

### Validation Controls
- [ ] Start button begins validation
- [ ] Cancel button stops validation
- [ ] Progress updates during scan

## Settings Screen

### Extraction Settings
- [ ] Postfixes input accepts comma-separated values
- [ ] Ignored files input accepts patterns
- [ ] Regex indicator shows for regex patterns
- [ ] "Ignore bad files" toggle works
- [ ] "Auto backup" toggle works
- [ ] Settings save immediately on change

### Personalization
- [ ] Theme mode dropdown (Light/Dark/System)
- [ ] Switching theme updates UI immediately
- [ ] Accent color picker changes UI accent color
- [ ] Language selector (Auto/EN/中文简体/中文繁體)
- [ ] Language change shows "restart required" indicator

### Update Settings
- [ ] "Check updates at startup" toggle works
- [ ] "Check for Updates" button checks GitHub
- [ ] Update notification shows if new version available

### Advanced Settings
- [ ] "Show Debug Log" toggle works
- [ ] "View Logs" button opens log viewer dialog
- [ ] Extraction path input and browse
- [ ] Backup path input and browse
- [ ] External tool path browse (for BA2 viewer)
- [ ] Paths validate and save correctly

### About Section
- [ ] Version number displays correctly
- [ ] Original author (KazumaKuun) credited
- [ ] Current maintainer (evildarkarchon) listed
- [ ] GitHub links work
- [ ] Nexus Mods link works
- [ ] Ko-fi link works
- [ ] License information (GPL-3.0) shown
- [ ] BSArch.exe attribution (MPL-2.0) shown

## Log Viewer Dialog (Debug)

### Opening/Closing
- [ ] Dialog opens when "View Logs" clicked in Settings
- [ ] Semi-transparent overlay covers main window
- [ ] Click overlay background closes dialog
- [ ] Click X button closes dialog

### Log Display
- [ ] Logs display with timestamps
- [ ] Log levels color-coded (red=ERROR, orange=WARN, white=INFO, gray=DEBUG/TRACE)
- [ ] Module/target shown for each entry
- [ ] Messages wrap correctly
- [ ] Monospace font used
- [ ] Scrollbar appears when needed

### Log Filtering
- [ ] "All" shows all log entries
- [ ] "ERROR" shows only errors
- [ ] "WARN" shows warnings and errors
- [ ] "INFO" shows info, warnings, errors
- [ ] "DEBUG" shows debug and above
- [ ] "TRACE" shows all including trace
- [ ] Entry count updates when filter changes

### Log Actions
- [ ] "Refresh" reloads logs from disk
- [ ] "Clear" empties log display
- [ ] "Copy" copies logs (shows info message - clipboard not implemented)

## Navigation & Layout

### Sidebar Navigation
- [ ] Extraction icon/label navigates to extraction screen
- [ ] Check Files icon/label navigates to validation screen
- [ ] Settings icon/label navigates to settings screen
- [ ] Active screen highlighted with accent color
- [ ] Hover effect on navigation items
- [ ] Settings button anchored at bottom

### Responsiveness
- [ ] Window resizes smoothly (min 800x500)
- [ ] Sidebar width adapts (220px @ >=1000px, 180px @ 800-999px)
- [ ] Font sizes scale with window width
- [ ] Content area uses available space
- [ ] No layout breaking at minimum size

### Animations
- [ ] Screen transitions fade and slide smoothly
- [ ] Button hover shows shadow and color change
- [ ] Progress bars animate smoothly
- [ ] Context menus fade in
- [ ] Navigation transitions smooth (200ms)

## Notifications & Dialogs

### Toast Notifications
- [ ] Success toasts show in green
- [ ] Error toasts show in red
- [ ] Warning toasts show in yellow
- [ ] Info toasts show in blue
- [ ] Toasts auto-dismiss after timeout
- [ ] Multiple toasts stack correctly

### Modal Dialogs
- [ ] Confirmation dialogs block interaction
- [ ] Primary button performs action
- [ ] Secondary button cancels
- [ ] Escape key closes dialog
- [ ] Click outside closes dialog (if dismissible)

## Theme System

### Light Theme
- [ ] Light background colors
- [ ] Dark text on light surface
- [ ] Accent color visible and consistent
- [ ] All UI elements readable
- [ ] Shadows and borders visible

### Dark Theme
- [ ] Dark background colors
- [ ] Light text on dark surface
- [ ] Accent color visible and consistent
- [ ] All UI elements readable
- [ ] Shadows and borders visible

### System Theme
- [ ] Detects OS theme preference (Windows)
- [ ] Switches theme when OS theme changes
- [ ] Consistent behavior on theme switch

## Error Handling

### User-Facing Errors
- [ ] Missing BSArch.exe shows clear error message
- [ ] Invalid config shows error with recovery suggestions
- [ ] Corrupted BA2 files reported clearly
- [ ] Extraction failures show helpful messages
- [ ] Validation errors display in results

### Error Recovery
- [ ] Application doesn't crash on errors
- [ ] Transient errors retried automatically
- [ ] Clear error messages shown to user
- [ ] Logs contain detailed error information

## Performance

### Large Datasets
- [ ] Scanning 100+ BA2 files completes in reasonable time
- [ ] File table scrolls smoothly with 100+ entries
- [ ] Extraction progress updates don't lag UI
- [ ] Log viewer handles 1000+ entries without lag

### Memory Usage
- [ ] Memory usage stays reasonable during scanning
- [ ] Memory usage stays reasonable during extraction
- [ ] No obvious memory leaks during extended use

## Cross-Platform (Windows Primary)

### Windows-Specific
- [ ] Registry detection finds default BA2 handler
- [ ] External tool auto-populates if handler found
- [ ] Toast notification shown for detection
- [ ] Console window hidden for BSArch.exe
- [ ] File paths with spaces handled correctly
- [ ] UNC paths handled correctly

## Configuration Persistence

### Settings Persistence
- [ ] Changed settings save immediately
- [ ] Settings persist across application restarts
- [ ] Invalid config loads with defaults
- [ ] Config migration works for older formats

### State Persistence
- [ ] Last used folder remembered
- [ ] Window size/position remembered (if implemented)
- [ ] Theme preference remembered
- [ ] Language preference remembered

## Accessibility

### Keyboard Navigation
- [ ] Tab key navigates between controls
- [ ] Enter activates buttons
- [ ] Escape closes dialogs
- [ ] Arrow keys navigate lists (if implemented)

### Visual Clarity
- [ ] All text readable in both themes
- [ ] Sufficient contrast ratios
- [ ] Icons clear and understandable
- [ ] Loading states clearly indicated

## Edge Cases

### Empty States
- [ ] Empty file table shows helpful message
- [ ] Empty log viewer shows helpful message
- [ ] Empty validation results handled
- [ ] No folder selected handled gracefully

### Invalid Input
- [ ] Invalid threshold values rejected
- [ ] Invalid regex patterns show error
- [ ] Invalid postfixes show error
- [ ] Invalid file paths show error

### Concurrent Operations
- [ ] Cannot start scan while extracting
- [ ] Cannot start extraction while scanning
- [ ] Pause/Resume/Cancel work correctly
- [ ] Multiple rapid clicks don't cause issues

## Known Limitations

- [ ] Drag-and-drop not implemented (Slint limitation)
- [ ] Clipboard copy not implemented (requires arboard crate)
- [ ] Internationalization not implemented (maintainer not fluent in Chinese)

## Test Environment

- **OS**: Windows 10/11 (primary), Windows WSL (development)
- **Screen Resolution**: Test at 1920x1080, 1366x768 (minimum supported)
- **Test Data**: Various BA2 files (textures, general, corrupted)
- **BSArch.exe Version**: [version bundled with release]

## Sign-Off

- [ ] All critical paths tested
- [ ] No blocking issues found
- [ ] Known issues documented
- [ ] Ready for release

**Tester**: ________________
**Date**: ________________
**Version Tested**: ________________
**Notes**: ________________
