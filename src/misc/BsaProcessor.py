import pathlib
from typing import Protocol

from PySide6.QtCore import QThread, Signal
from PySide6.QtWidgets import QApplication

from misc.Config import Config, LogLevel, cfg
from misc.Utilities import is_ignored, num_files_in_ba2, scan_for_ba2
from model.PreviewTableModel import FileEntry


class ParentProtocol(Protocol):
    """Protocol defining the interface that BsaProcessor expects from its parent."""
    failed: set[pathlib.Path]
    file_data: list[FileEntry]


class BsaProcessor(QThread):
    done_processing = Signal(list)

    def __init__(self, mod_folder: str, _parent: ParentProtocol) -> None:
        super().__init__()

        self._path: str = mod_folder
        self._parent: ParentProtocol = _parent

    def run(self) -> None:
        ba2_paths: list[str] = scan_for_ba2(self._path, cfg.get(Config.postfixes))

        num_failed: int = 0
        num_ignored: int = 0
        num_success: int = 0
        temp: list[FileEntry] = []
        # Populate ba2 files and their properties
        for f in ba2_paths:
            dir_: str = pathlib.Path(pathlib.Path(f).parent).name
            name: str = pathlib.Path(f).name
            size: int = pathlib.Path(f).stat().st_size
            num_files: int = num_files_in_ba2(f)
            # Auto ignore the blacklisted file if set so
            if is_ignored(f):
                num_ignored += 1
                QApplication.instance().log_view.add_log(f"Ignoring {f}", LogLevel.INFO) # pyright: ignore[reportAttributeAccessIssue]
            elif num_files == -1:
                num_failed += 1
                if cfg.ignore_bad_files:
                    self._parent.failed.add(pathlib.Path(f).resolve())
            else:
                num_success += 1
                temp.append(FileEntry(name, size, num_files, dir_, f))

        temp = sorted(temp, key=lambda entry: entry.file_size)
        self._parent.file_data = temp
        self.done_processing.emit([self._parent, num_success, num_failed, num_ignored])
