import os.path
import pathlib
import subprocess
from typing import Any

from humanize import naturalsize
from misc.BsaExtractor import BsaExtractor
from misc.BsaProcessor import BsaProcessor
from misc.Config import cfg
from misc.Utilities import check_latest_version, parse_size
from model.PreviewTableModel import FileEntry, PreviewTableModel
from prefab.InfoBar import show_result_toast, show_update_available
from prefab.MessageBox import auto_not_available
from PySide6.QtCore import QEvent, QModelIndex, QPoint, QSortFilterProxyModel, Qt
from PySide6.QtGui import QDragEnterEvent, QDragLeaveEvent, QDropEvent, QResizeEvent
from PySide6.QtWidgets import (
    QAbstractItemView,
    QApplication,
    QBoxLayout,
    QFileDialog,
    QFrame,
    QHBoxLayout,
    QHeaderView,
    QVBoxLayout,
    QWidget,
)
from qfluentwidgets import (
    Action,
    BodyLabel,
    LineEdit,
    PrimaryPushButton,
    RoundMenu,
    StrongBodyLabel,
    SubtitleLabel,
    TableView,
    TogglePushButton,
    ToolButton,
    ToolTipFilter,
)
from qfluentwidgets import FluentIcon as Fi


class MainScreen(QFrame):
    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent=parent)
        self.setObjectName("MainScreen")
        self.layout: QVBoxLayout = QVBoxLayout(self)

        # Subsection Setup
        self.setup_title: SubtitleLabel = SubtitleLabel(self.tr("Extraction setup"), self)
        self.setup_layout: QHBoxLayout = QHBoxLayout()

        # Folder chooser
        self.folder_layout: QVBoxLayout = QVBoxLayout()
        self.folder_layout_inner: QHBoxLayout = QHBoxLayout()
        self.folder_label: StrongBodyLabel = StrongBodyLabel(self.tr("Fallout 4 mod folder"), self)
        self.folder_input: LineEdit = LineEdit(self)
        self.folder_input.returnPressed.connect(self.__process_folder)
        self.folder_button: ToolButton = ToolButton(Fi.FOLDER, self)

        # Threshold
        self.threshold_layout: QVBoxLayout = QVBoxLayout()
        self.threshold_layout_inner: QHBoxLayout = QHBoxLayout()
        self.threshold_label: StrongBodyLabel = StrongBodyLabel(self.tr("Extraction size threshold"), self)
        self.threshold_input: LineEdit = LineEdit(self)
        self.threshold_button: TogglePushButton = TogglePushButton(Fi.SYNC, self.tr("Auto"), self)

        # Start button
        self.start_layout: QHBoxLayout = QHBoxLayout()
        self.start_button: PrimaryPushButton = PrimaryPushButton(Fi.SEND_FILL, self.tr("Start"), self)

        # Subsection Preview
        self.above_table_layout: QHBoxLayout = QHBoxLayout()
        self.preview_title: SubtitleLabel = SubtitleLabel(self.tr("Preview"), self)
        self.preview_text: BodyLabel = BodyLabel("", self)

        # self.preview_progress: ProgressBar = ProgressBar()

        # File table
        self.preview_table: TableView = TableView(self)

        # Hint on top of the preview table
        self.preview_hint_layout: QBoxLayout = QBoxLayout(QBoxLayout.Direction.LeftToRight, self.preview_table)
        self.preview_hint: SubtitleLabel = SubtitleLabel(self.tr("Select or drag 'n drop a folder here to get started"), self)
        self.preview_hint.setAlignment(Qt.AlignmentFlag.AlignCenter)

        # File related
        self.processor: BsaProcessor | None = None
        self.extractor: BsaExtractor | None = None
        self.file_data: list[FileEntry] = []
        self.failed: set[pathlib.Path] = set()

        self.table_ready: bool = False
        self.persistent_tooltip: Any | None = None

        self.__setup_interface()
        self.__check_update()

    def __setup_interface(self) -> None:
        self.layout.setAlignment(Qt.AlignmentFlag.AlignTop)

        # Setup section
        self.layout.addWidget(self.setup_title, 0, Qt.AlignmentFlag.AlignLeft)

        # Folder chooser
        self.folder_layout_inner.setAlignment(Qt.AlignmentFlag.AlignLeft)
        self.folder_input.setPlaceholderText(self.tr("Fallout 4 mod folder"))
        # Open a folder chooser when clicking
        self.folder_button.clicked.connect(self.__open_folder)

        self.folder_layout_inner.addWidget(self.folder_input)
        self.folder_layout_inner.addWidget(self.folder_button)
        self.folder_layout.addWidget(self.folder_label)
        self.folder_layout.addLayout(self.folder_layout_inner)

        # Threshold
        self.threshold_layout_inner.setAlignment(Qt.AlignmentFlag.AlignLeft)
        self.threshold_input.setPlaceholderText(self.tr("Threshold"))
        self.threshold_input.textEdited.connect(self.__threshold_changed)

        # Perform auto threshold calculation and disable user input box
        self.threshold_button.clicked.connect(self.__auto_toggled)
        self.threshold_button.setToolTip(
            self.tr("Automatically attempts to determine a\nthreshold so just enough ba2 are\nextracted to get under the ba2 limit")
        )
        self.threshold_button.installEventFilter(ToolTipFilter(self.threshold_button))

        self.threshold_layout_inner.addWidget(self.threshold_input)
        self.threshold_layout_inner.addWidget(self.threshold_button)
        self.threshold_layout_inner.addStretch(1)
        self.threshold_layout.addWidget(self.threshold_label)
        self.threshold_layout.addLayout(self.threshold_layout_inner)

        # Main setup layout
        self.setup_layout.addLayout(self.folder_layout, stretch=1)
        self.setup_layout.addSpacing(15)
        self.setup_layout.addLayout(self.threshold_layout)
        self.layout.addLayout(self.setup_layout)

        self.layout.addSpacing(10)

        # Start button
        self.start_button.setMaximumWidth(300)
        self.start_button.setEnabled(False)
        self.start_button.clicked.connect(self.__extract_files)
        self.start_layout.addWidget(self.start_button)
        self.layout.addLayout(self.start_layout)

        # Preview section
        self.above_table_layout.addWidget(self.preview_title)
        self.above_table_layout.addStretch(1)
        self.above_table_layout.addWidget(self.preview_text)
        self.layout.addLayout(self.above_table_layout)

        self.__setup_table()

        # Hide the progress bar in the beginning
        # sp = self.preview_progress.sizePolicy()
        # sp.setRetainSizeWhenHidden(True)
        # self.preview_progress.setSizePolicy(sp)
        # self.preview_progress.setHidden(True)
        # self.layout.addWidget(self.preview_progress)

        # Leave some space for the title bar
        self.layout.setContentsMargins(60, 42, 60, 10)

        # Drag and drop
        self.setLayout(self.layout)
        self.setAcceptDrops(True)

    def __setup_table(self) -> None:
        # Tableview configs
        self.preview_table.setBorderVisible(True)
        self.preview_table.setBorderRadius(8)

        self.preview_table.setWordWrap(False)
        vh: QHeaderView = QHeaderView(Qt.Orientation.Vertical)
        vh.hide()
        self.preview_table.setVerticalHeader(vh)

        # Context menu and double click
        self.preview_table.setContextMenuPolicy(Qt.ContextMenuPolicy.CustomContextMenu)
        self.preview_table.customContextMenuRequested.connect(self.__table_custom_menu)
        self.preview_table.doubleClicked.connect(self.__table_double_click)

        sp = self.preview_table.sizePolicy()
        sp.setRetainSizeWhenHidden(True)
        self.preview_table.setSizePolicy(sp)

        self.preview_table.setSelectionMode(QAbstractItemView.SelectionMode.SingleSelection)

        self.layout.addWidget(self.preview_table)

        # Proxy model used by the table
        proxy_model = QSortFilterProxyModel()
        proxy_model.setSortRole(Qt.ItemDataRole.UserRole)
        proxy_model.setSortCaseSensitivity(Qt.CaseSensitivity.CaseInsensitive)
        self.preview_table.setModel(proxy_model)
        self.preview_table.horizontalHeader().setStretchLastSection(True)
        # Sorting
        self.preview_table.horizontalHeader().setSortIndicator(1, Qt.SortOrder.AscendingOrder)
        self.preview_table.setSortingEnabled(True)

        # Add the hint to the center of the table
        self.preview_hint_layout.addWidget(self.preview_hint, 0, Qt.AlignmentFlag.AlignCenter)

    def __process_folder(self) -> None:
        selected_folder: str = self.folder_input.text()
        self.start_button.setDisabled(True)
        self.failed.clear()

        # Only process if the folder selected is not empty
        if selected_folder and pathlib.Path(selected_folder).is_dir():
            cfg.set(cfg.saved_dir, selected_folder)
            self.folder_button.setDisabled(True)
            # Animate the progress bar
            self.processor = BsaProcessor(selected_folder, self)

            self.preview_hint.setHidden(True)

            self.processor.done_processing.connect(show_result_toast)
            self.processor.finished.connect(self.__done_loading_ba2)
            self.processor.start()

    def __open_folder(self) -> None:
        self.folder_input.setText(
            QFileDialog.getExistingDirectory(
                self,
                self.tr("Open your Fallout 4 mod folder"),
                dir=cfg.get(cfg.saved_dir),
                options=QFileDialog.Option.ShowDirsOnly | QFileDialog.Option.DontResolveSymlinks,
            )
        )
        self.__process_folder()

    def __extract_files(self) -> None:
        self.preview_table.setDisabled(True)
        self.start_button.setDisabled(True)

        self.extractor = BsaExtractor(self)
        self.extractor.done_processing.connect(self.__done_extracting)
        self.preview_hint.setText(self.tr("Extracting files..."))
        self.preview_hint.setHidden(False)

        self.extractor.start()

    def __auto_toggled(self) -> None:
        # Disable threshold input if "Auto" is enabled
        self.threshold_input.setDisabled(self.threshold_input.isEnabled())
        if self.threshold_button.isChecked():
            self.__determine_threshold()
        else:
            self.threshold_input.clear()
            self.__refresh_table(self.file_data)

    def __refresh_table(self, data: list[FileEntry]) -> None:
        model: PreviewTableModel = PreviewTableModel(data)
        self.preview_table.model().setSourceModel(model)

        for i in range(self.preview_table.model().rowCount()):
            self.preview_table.setRowHidden(i, False)

        self.folder_button.setDisabled(False)
        self.table_ready = True
        self.__update_preview_text()
        self.__check_start_ready()

    def __update_ignored(self) -> None:
        if len(self.failed) > 0:
            temp: set[Any] = set(cfg.get(cfg.ignored))
            temp.update(self.failed)
            cfg.set(cfg.ignored, list(temp))
            # Update the ignored items accordingly
            QApplication.instance().ignore_changed.emit()

    def __done_loading_ba2(self) -> None:
        if self.threshold_button.isChecked():
            self.__determine_threshold()
        else:
            self.__refresh_table(self.file_data)
        self.__adjust_column_size()

        self.__update_ignored()
        del self.processor

    def __done_extracting(self, results: list[int]) -> None:
        tmp: list[Any] = [self]
        tmp.extend(results)
        show_result_toast(tmp, "extract")

        self.preview_table.setDisabled(False)
        self.preview_table.repaint()

        self.__update_ignored()
        self.preview_text.setText("")
        self.preview_hint.setText(self.tr("Select or drag 'n drop a folder here to get started"))
        self.preview_hint.setHidden(True)

    def __adjust_column_size(self) -> None:
        total_width: int = self.preview_table.width()
        fs_col_width: int = 95
        num_col_width: int = 81

        self.preview_table.setColumnWidth(0, int((total_width - fs_col_width - num_col_width) * 0.45))
        self.preview_table.setColumnWidth(1, fs_col_width)
        self.preview_table.setColumnWidth(2, num_col_width)

    # Resize the columns of the table automatically on resize
    def resizeEvent(self, event: QResizeEvent) -> None:
        self.__adjust_column_size()

    def __threshold_changed(self) -> None:
        text: str = self.threshold_input.text()
        if not text:
            self.__refresh_table(self.file_data)
            return
        threshold_byte: int = parse_size(text)
        filtered: list[FileEntry] = self.__get_filtered_files(threshold_byte)
        self.__refresh_table(filtered)

    def __table_double_click(self, item_idx: QModelIndex) -> None:
        if not item_idx.isValid():
            return

        raw_idx: QModelIndex = self.preview_table.model().mapToSource(item_idx)
        raw_data: list[FileEntry] = self.preview_table.model().sourceModel().raw_data()
        data: FileEntry = raw_data[raw_idx.row()]
        self.__open_ba2_ext(data.full_path)

    def __table_custom_menu(self, pos: QPoint) -> None:
        item_idx: QModelIndex = self.preview_table.indexAt(pos)
        if not item_idx.isValid():
            return

        menu: RoundMenu = RoundMenu()

        # Add actions one by one, Action inherits from QAction and accepts icons of type FluentIconBase
        raw_idx: QModelIndex = self.preview_table.model().mapToSource(item_idx)
        raw_data: list[FileEntry] = self.preview_table.model().sourceModel().raw_data()
        data: FileEntry = raw_data[raw_idx.row()]

        menu.addAction(Action(Fi.REMOVE_FROM, self.tr("Ignore"), triggered=lambda: self.__ignore_file(data.full_path, raw_idx.row())))
        menu.addAction(Action(Fi.LINK, self.tr("Open"), triggered=lambda: self.__open_ba2_ext(data.full_path)))

        menu.exec_(self.preview_table.viewport().mapToGlobal(pos))

    def __check_start_ready(self) -> None:
        table_nonempty: bool = self.preview_table.model().rowCount() > 0
        self.start_button.setEnabled(self.table_ready and table_nonempty)

    def __determine_threshold(self) -> None:
        if len(self.file_data) <= 235:
            if self.table_ready:
                auto_not_available(self)
            return

        threshold: int = self.file_data[-235].file_size
        self.threshold_input.setText(naturalsize(threshold))

        filtered: list[FileEntry] = self.__get_filtered_files(threshold)
        self.__refresh_table(filtered)

    def __get_filtered_files(self, threshold_byte: int) -> list[FileEntry]:
        if threshold_byte != -1:
            # Persist the size info
            cfg.set(cfg.saved_threshold, threshold_byte)
            return [entry for entry in self.file_data if entry.file_size <= threshold_byte]
        return []

    def __ignore_file(self, file_name: str, idx: int) -> None:
        temp: set[Any] = set(cfg.get(cfg.ignored))
        temp.add(os.path.abspath(file_name))
        cfg.set(cfg.ignored, list(temp))
        # Update the ignored items accordingly
        QApplication.instance().ignore_changed.emit()

        self.preview_table.model().sourceModel().removeRow(idx)
        self.__update_preview_text()

    def __open_ba2_ext(self, file_path: str) -> None:
        if pathlib.Path(file_path).is_file():
            args: list[str] = [
                cfg.get(cfg.ext_ba2_exe),
                file_path,
            ]
            subprocess.run(args)

    def __update_preview_text(self) -> None:
        if self.preview_table.model().rowCount() > 0:
            curr_data: list[FileEntry] = self.preview_table.model().sourceModel().files
            self.preview_text.setText(
                self.tr("Total files: {0}, total size: {1}, extracted file count: {2}").format(
                    len(curr_data), naturalsize(sum([x.file_size for x in curr_data])), sum([x.num_files for x in curr_data])
                ),
            )

    def __check_update(self) -> None:
        if cfg.get(cfg.check_update_at_start_up):
            latest: str | None = check_latest_version()
            if latest:
                show_update_available(self, latest)

    # Drag and drop
    def dragEnterEvent(self, event: QDragEnterEvent) -> None:
        self.preview_hint.setText(self.tr("Drop your Fallout 4 mod folder here"))
        if event.mimeData().hasUrls():
            event.accept()
        else:
            event.ignore()

    def dragLeaveEvent(self, event: QDragLeaveEvent) -> None:
        self.preview_hint.setText(self.tr("Select a folder to get started"))

    def dropEvent(self, event: QDropEvent) -> None:
        files: list[str] = [u.toLocalFile() for u in event.mimeData().urls()]
        if len(files) == 1 and pathlib.Path(files[0]).is_dir():
            self.folder_input.setText(files[0])
            self.__process_folder()
