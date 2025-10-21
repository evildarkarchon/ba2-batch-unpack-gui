from dataclasses import dataclass
from typing import Literal

from PySide6.QtWidgets import QApplication
from qfluentwidgets import HyperlinkButton, InfoBar, InfoBarIcon, InfoBarPosition, PushButton

from misc.Config import NEXUS_URL, VERSION, cfg
from prefab.MessageBox import show_failed_files

OperationType = Literal["scan", "check", "extract"]


@dataclass
class MessageConfig:
    """Configuration for different operation types."""
    verb: str
    fail_title: str
    success_title: str
    success_message: str
    count_message: str
    supports_details: bool


def _get_message_config(op_type: OperationType) -> MessageConfig:
    """
    Get message configuration for a given operation type.

    Returns:
        MessageConfig: Configuration object containing all text for the operation type.
    """
    configs = {
        "scan": MessageConfig(
            verb=QApplication.translate("InfoBar", "scanning"),
            fail_title=QApplication.translate("InfoBar", "Failed to load some files"),
            success_title=QApplication.translate("InfoBar", "Ready"),
            success_message=QApplication.translate("InfoBar", "Finished scanning ba2. Processed {0} files."),
            count_message=QApplication.translate("InfoBar", "\nProcessed {0} files."),
            supports_details=True,
        ),
        "check": MessageConfig(
            verb=QApplication.translate("InfoBar", "checking"),
            fail_title=QApplication.translate("InfoBar", "Some files did not pass the check"),
            success_title=QApplication.translate("InfoBar", "All good"),
            success_message=QApplication.translate("InfoBar", "Finished checking ba2. Checked {0} files."),
            count_message=QApplication.translate("InfoBar", "\nChecked {0} files."),
            supports_details=False,
        ),
        "extract": MessageConfig(
            verb=QApplication.translate("InfoBar", "extracting"),
            fail_title=QApplication.translate("InfoBar", "Failed to extract some files"),
            success_title=QApplication.translate("InfoBar", "All done"),
            success_message=QApplication.translate("InfoBar", "Finished extracting ba2. Extracted {0} files."),
            count_message=QApplication.translate("InfoBar", "\nExtracted {0} files."),
            supports_details=True,
        ),
    }
    return configs.get(op_type, configs["extract"])


def _build_fail_message(config: MessageConfig, num_fail: int, num_success: int, op_type: OperationType) -> str:
    """
    Build the failure message based on configuration.

    Returns:
        str: Formatted failure message with file counts and appropriate suffix.
    """
    auto_ignore = cfg.get(cfg.ignore_bad_files)

    # Base message
    message = QApplication.translate("InfoBar", "Finished {0} ba2. Could not open {1} files").format(
        config.verb, num_fail
    )

    # Add suffix based on operation type and settings
    if op_type == "check":
        message += QApplication.translate("InfoBar", ". Please check the output above.")
    elif auto_ignore:
        message += QApplication.translate("InfoBar", " and they will be ignored in the future.")
    else:
        message += QApplication.translate("InfoBar", ".")

    # Add count message
    message += config.count_message.format(num_success)

    return message


def _build_success_message(config: MessageConfig, num_success: int, results: list) -> str:
    """
    Build the success message based on configuration.

    Returns:
        str: Formatted success message with file counts and optional ignored count.
    """
    message = config.success_message.format(num_success)

    # Add ignored files count for scan operations
    if len(results) > 3:
        num_ignored = results[3]
        if num_ignored > 0:
            message += QApplication.translate("InfoBar", " Skipped {0} ignored files.").format(num_ignored)

    return message


def show_result_toast(results: list, _type: OperationType = "scan") -> None:
    """Show a toast notification with operation results."""
    num_success = results[1]
    num_fail = results[2]
    parent = results[0]

    config = _get_message_config(_type)

    if num_fail > 0:
        # Show warning for failures
        fail_message = _build_fail_message(config, num_fail, num_success, _type)
        warning_info = InfoBar(
            icon=InfoBarIcon.WARNING,
            title=config.fail_title,
            content=fail_message,
            duration=-1,
            position=InfoBarPosition.BOTTOM,
            parent=parent,
        )

        # Add details button for supported operation types
        if config.supports_details:
            more_info_button = PushButton(QApplication.translate("InfoBar", "Details"), warning_info)
            more_info_button.clicked.connect(lambda: show_failed_files(parent))
            warning_info.addWidget(more_info_button)

        warning_info.show()
    else:
        # Show success message
        success_message = _build_success_message(config, num_success, results)
        InfoBar.success(
            title=config.success_title,
            content=success_message,
            duration=5000,
            position=InfoBarPosition.BOTTOM,
            parent=parent,
        )


def show_update_available(parent, new_ver: str) -> None:
    """Show an info bar notifying the user of an available update."""
    if "v" in new_ver:
        new_ver = new_ver[1:]
    update_info = InfoBar(
        icon=InfoBarIcon.INFORMATION,
        title=QApplication.translate("InfoBar", "Update available"),
        content=QApplication.translate("InfoBar",
                                       "A new version of Unpackrr is available.\n"
                                       "Current: {0}, latest: {1}").format(VERSION, new_ver),
        duration=-1,
        position=InfoBarPosition.BOTTOM,
        parent=parent,
    )
    download_button = HyperlinkButton(NEXUS_URL+"?tab=files", QApplication.translate("InfoBar", "Download"))
    update_info.addWidget(download_button)
    update_info.show()
