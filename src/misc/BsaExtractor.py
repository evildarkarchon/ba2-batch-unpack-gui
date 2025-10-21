import pathlib
import shutil
from typing import TYPE_CHECKING, cast

from PySide6.QtCore import QModelIndex, QSortFilterProxyModel, QThread, Signal

from misc.Config import cfg
from misc.Utilities import extract_ba2, resource_path

if TYPE_CHECKING:
    from PySide6.QtWidgets import QTableView

    from model.PreviewTableModel import PreviewTableModel


class BsaExtractor(QThread):
    done_processing = Signal(list)

    def __init__(self, parent) -> None:  # noqa: ANN001
        super().__init__()
        self._parent = parent

    def run(self) -> None:
        table: QTableView = self._parent.preview_table
        # progress: ProgressBar = self._parent.preview_progress
        failed: set[pathlib.Path] = self._parent.failed

        # progress.show()
        # progress.setMaximum(table.model().rowCount())

        ok_count: int = 0
        failed_count: int = 0

        # Cast the model to QSortFilterProxyModel to access sourceModel()
        proxy_model: QSortFilterProxyModel = cast("QSortFilterProxyModel", table.model())
        source_model: PreviewTableModel = cast("PreviewTableModel", proxy_model.sourceModel())

        for table_idx in range(proxy_model.rowCount()):
            path: str = source_model.raw_data()[table_idx].full_path
            if extract_ba2(path, resource_path("bin/BSArch.exe")) == -1:
                if cfg.get(cfg.ignore_bad_files):
                    failed.add(pathlib.Path(path).resolve())
                # Highlight the failed files in the table
                source_idx: QModelIndex = proxy_model.mapToSource(proxy_model.index(table_idx, 0))
                source_model.add_bad_file(source_idx.row())
                # progress.error()
                failed_count += 1
            else:
                # Back up the file if user requests so
                if cfg.get(cfg.auto_backup):
                    cfg_path: str = cfg.get(cfg.backup_path)
                    backup_path: pathlib.Path
                    if cfg_path:
                        backup_path = cfg_path if pathlib.Path(cfg_path).is_absolute() else pathlib.Path(path).parent / cfg_path
                    else:
                        backup_path = pathlib.Path(path).parent / "backup"

                    if not pathlib.Path(backup_path).is_dir():
                        pathlib.Path(backup_path).mkdir(parents=True)
                    shutil.move(path, pathlib.Path(backup_path) / pathlib.Path(path).name)
                else:
                    pathlib.Path(path).unlink()
                # Remove the row from the preview
                table.hideRow(table_idx)
                ok_count += 1
            # progress.setValue(progress.value() + 1)

        self.done_processing.emit([ok_count, failed_count])
