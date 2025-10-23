//! Notification and dialog management for the UI
//!
//! This module provides helper functions for showing toast notifications and modal dialogs.
//! It integrates with the Slint UI components defined in main.slint.

use crate::ui::{MainWindow, NotificationType};
use slint::{ComponentHandle, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::rc::Rc;

/// Toast notification data structure
#[derive(Clone)]
pub struct ToastData {
    pub message: String,
    pub notification_type: NotificationType,
    pub show: bool,
}

impl ToastData {
    /// Create a new success toast
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Success,
            show: true,
        }
    }

    /// Create a new error toast
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Error,
            show: true,
        }
    }

    /// Create a new warning toast
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Warning,
            show: true,
        }
    }

    /// Create a new info toast
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Info,
            show: true,
        }
    }

    /// Convert to Slint's tuple format (message, show, type)
    /// Note: The order must match the Slint anonymous struct field order
    fn to_slint_tuple(&self) -> (SharedString, bool, NotificationType) {
        (
            self.message.clone().into(),
            self.show,
            self.notification_type,
        )
    }
}

/// Show a toast notification
///
/// This adds a toast to the notification queue. Toasts will auto-dismiss after a timeout.
/// Uses Slint's Timer API to ensure thread-safety and prevent UI blocking.
///
/// # Example
///
/// ```ignore
/// use unpackrr::ui::notifications::{show_toast, ToastData};
///
/// // Assuming you have a MainWindow instance
/// show_toast(&window, ToastData::success("Operation completed!"));
/// ```
pub fn show_toast(window: &MainWindow, toast: ToastData) {
    let current_toasts = window.get_toasts();
    let mut toasts_vec = Vec::new();

    // Copy existing toasts
    for i in 0..current_toasts.row_count() {
        if let Some(toast_tuple) = current_toasts.row_data(i) {
            toasts_vec.push(toast_tuple);
        }
    }

    // Add new toast
    toasts_vec.push(toast.to_slint_tuple());

    // Calculate index before moving the vector
    let toast_index = toasts_vec.len() - 1;

    // Update UI
    let new_model = Rc::new(VecModel::from(toasts_vec));
    window.set_toasts(ModelRc::from(new_model));

    // Schedule auto-dismiss after 5 seconds using Slint's Timer
    // This is thread-safe and runs on the event loop
    let window_weak = window.as_weak();

    let timer = Timer::default();
    timer.start(
        TimerMode::SingleShot,
        std::time::Duration::from_secs(5),
        move || {
            if let Some(window) = window_weak.upgrade() {
                dismiss_toast(&window, toast_index);
            }
        },
    );
}

/// Dismiss a toast notification by index
fn dismiss_toast(window: &MainWindow, index: usize) {
    let current_toasts = window.get_toasts();
    let mut toasts_vec = Vec::new();

    for i in 0..current_toasts.row_count() {
        if i != index {
            if let Some(toast_tuple) = current_toasts.row_data(i) {
                toasts_vec.push(toast_tuple);
            }
        }
    }

    let new_model = Rc::new(VecModel::from(toasts_vec));
    window.set_toasts(ModelRc::from(new_model));
}

/// Dialog configuration
pub struct DialogConfig {
    pub title: String,
    pub message: String,
    pub dialog_type: NotificationType,
    pub primary_button: String,
    pub secondary_button: Option<String>,
}

impl DialogConfig {
    /// Create a new info dialog
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            dialog_type: NotificationType::Info,
            primary_button: "OK".to_string(),
            secondary_button: None,
        }
    }

    /// Create a new success dialog
    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            dialog_type: NotificationType::Success,
            primary_button: "OK".to_string(),
            secondary_button: None,
        }
    }

    /// Create a new error dialog
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            dialog_type: NotificationType::Error,
            primary_button: "OK".to_string(),
            secondary_button: None,
        }
    }

    /// Create a new warning dialog
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            dialog_type: NotificationType::Warning,
            primary_button: "OK".to_string(),
            secondary_button: None,
        }
    }

    /// Create a confirmation dialog
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            dialog_type: NotificationType::Warning,
            primary_button: "Yes".to_string(),
            secondary_button: Some("No".to_string()),
        }
    }

    /// Set the primary button text
    pub fn with_primary_button(mut self, text: impl Into<String>) -> Self {
        self.primary_button = text.into();
        self
    }

    /// Set the secondary button text
    pub fn with_secondary_button(mut self, text: impl Into<String>) -> Self {
        self.secondary_button = Some(text.into());
        self
    }
}

/// Show a modal dialog
///
/// This displays a modal dialog that blocks interaction with the rest of the UI.
///
/// # Example
///
/// ```ignore
/// use unpackrr::ui::notifications::{show_dialog, DialogConfig};
///
/// // Assuming you have a MainWindow instance
/// let config = DialogConfig::confirm("Delete File", "Are you sure you want to delete this file?");
/// show_dialog(&window, config);
/// ```
pub fn show_dialog(window: &MainWindow, config: DialogConfig) {
    window.set_dialog_title(config.title.into());
    window.set_dialog_message(config.message.into());
    window.set_dialog_type(config.dialog_type);
    window.set_dialog_primary_button(config.primary_button.into());
    window.set_dialog_secondary_button(
        config
            .secondary_button
            .unwrap_or_default()
            .into(),
    );
    window.set_show_dialog(true);
}

/// Hide the currently displayed dialog
pub fn hide_dialog(window: &MainWindow) {
    window.set_show_dialog(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_data_success() {
        let toast = ToastData::success("Test message");
        assert_eq!(toast.message, "Test message");
        assert!(matches!(
            toast.notification_type,
            NotificationType::Success
        ));
        assert!(toast.show);
    }

    #[test]
    fn test_toast_data_error() {
        let toast = ToastData::error("Error message");
        assert_eq!(toast.message, "Error message");
        assert!(matches!(toast.notification_type, NotificationType::Error));
        assert!(toast.show);
    }

    #[test]
    fn test_toast_data_warning() {
        let toast = ToastData::warning("Warning message");
        assert_eq!(toast.message, "Warning message");
        assert!(matches!(
            toast.notification_type,
            NotificationType::Warning
        ));
        assert!(toast.show);
    }

    #[test]
    fn test_toast_data_info() {
        let toast = ToastData::info("Info message");
        assert_eq!(toast.message, "Info message");
        assert!(matches!(toast.notification_type, NotificationType::Info));
        assert!(toast.show);
    }

    #[test]
    fn test_dialog_config_info() {
        let config = DialogConfig::info("Title", "Message");
        assert_eq!(config.title, "Title");
        assert_eq!(config.message, "Message");
        assert!(matches!(config.dialog_type, NotificationType::Info));
        assert_eq!(config.primary_button, "OK");
        assert!(config.secondary_button.is_none());
    }

    #[test]
    fn test_dialog_config_confirm() {
        let config = DialogConfig::confirm("Confirm", "Are you sure?");
        assert_eq!(config.title, "Confirm");
        assert_eq!(config.message, "Are you sure?");
        assert!(matches!(config.dialog_type, NotificationType::Warning));
        assert_eq!(config.primary_button, "Yes");
        assert_eq!(config.secondary_button, Some("No".to_string()));
    }

    #[test]
    fn test_dialog_config_builder() {
        let config = DialogConfig::info("Title", "Message")
            .with_primary_button("Continue")
            .with_secondary_button("Cancel");

        assert_eq!(config.primary_button, "Continue");
        assert_eq!(config.secondary_button, Some("Cancel".to_string()));
    }
}
