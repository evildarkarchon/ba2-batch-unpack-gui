from typing import Any, NamedTuple

from humanize import naturalsize
from PySide6 import QtCore
from PySide6.QtCore import QModelIndex, Qt
from PySide6.QtGui import QBrush, QColor


class FileEntry(NamedTuple):
    file_name: str
    file_size: int
    num_files: int
    dir_name: str
    full_path: str


class PreviewTableModel(QtCore.QAbstractTableModel):
    # Data is assumed to be sorted according to their file size
    def __init__(self, data: list[FileEntry]) -> None:
        super().__init__()
        self.files: list[FileEntry] = data
        self.bad_ba2_idx: list[int] = []
        self.horizontalHeader = [self.tr("File Name"), self.tr("File Size"), self.tr("# Files"), self.tr("Mod")]

    def data(self, index: QModelIndex, role: Qt.ItemDataRole = Qt.ItemDataRole.DisplayRole) -> Any | None:
        if not index.isValid():
            return None

        if role in {Qt.ItemDataRole.DisplayRole, Qt.ItemDataRole.EditRole, Qt.ItemDataRole.UserRole}:
            # Assumes the following layout
            # File Name, File Size, # Files, Path, Ignored
            if index.column() == 0:
                return self.files[index.row()].file_name
            if index.column() == 1:
                if role == Qt.ItemDataRole.UserRole:
                    return self.files[index.row()].file_size
                return naturalsize(self.files[index.row()].file_size)
            if index.column() == 2:
                return self.files[index.row()].num_files
            if index.column() == 3:
                return self.files[index.row()].dir_name
            if index.column() == 4:
                return self.files[index.row()].full_path
        elif role == Qt.ItemDataRole.BackgroundRole:
            if index.row() in self.bad_ba2_idx:
                # Dark red
                return QBrush(QColor(139, 0, 0))
        return None

    def raw_data(self) -> list[FileEntry]:
        return self.files

    def add_bad_file(self, index: int) -> None:
        self.bad_ba2_idx.append(index)

    def size_at(self, index: int) -> int:
        if len(self.files) > index:
            return self.files[index].file_size
        return -1

    def removeRow(self, row: int, parent: QModelIndex | None = None) -> bool:
        if parent is None:
            parent = QModelIndex()
        self.beginRemoveRows(parent, row, row)
        self.files.pop(row)
        self.endRemoveRows()
        return True

    def flags(self, index: QModelIndex) -> Qt.ItemFlag | None:
        if not index.isValid():
            return None

        return Qt.ItemFlag.ItemIsEnabled | Qt.ItemFlag.ItemIsSelectable

    def rowCount(self, _parent: QModelIndex | None = None) -> int:
        # The length of the outer list.
        return len(self.files)

    def columnCount(self, _parent: QModelIndex | None = None) -> int:
        return 4

    def headerData(self, section: int, orientation: Qt.Orientation, role: Qt.ItemDataRole = Qt.ItemDataRole.DisplayRole) -> Any | None:
        if role == Qt.ItemDataRole.DisplayRole and orientation == QtCore.Qt.Orientation.Horizontal:
            return self.horizontalHeader[section]
        return None
