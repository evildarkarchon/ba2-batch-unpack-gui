from typing import cast

from PySide6.QtCore import QSize, Qt, Signal
from PySide6.QtGui import QIcon
from PySide6.QtWidgets import QHBoxLayout, QLabel, QSizePolicy, QWidget
from qfluentwidgets import (
    ConfigItem,
    ExpandSettingCard,
    FluentIconBase,
    InfoBarIcon,
    LineEdit,
    PushButton,
    TeachingTip,
    TeachingTipTailPosition,
    ToolButton,
    ToolTipFilter,
    qconfig,
)
from qfluentwidgets import FluentIcon as Fi


class IgnoredItem(QWidget):
    """Ignored item"""

    removed = Signal(QWidget)

    def __init__(self, ignored: str, parent=None) -> None:  # noqa: ANN001
        super().__init__(parent=parent)
        self.ignored = ignored
        self.item_layout = QHBoxLayout(self)
        self.ignored_label = QLabel(ignored, self)
        self.remove_button = ToolButton(Fi.CLOSE, self)

        self.remove_button.setFixedSize(39, 29)
        self.remove_button.setIconSize(QSize(12, 12))

        self.setFixedHeight(53)
        self.setSizePolicy(QSizePolicy.Policy.Ignored, QSizePolicy.Policy.Fixed)
        self.item_layout.setContentsMargins(48, 0, 60, 0)

        self.item_layout.addWidget(self.ignored_label, 0, Qt.AlignmentFlag.AlignLeft)
        self.item_layout.addSpacing(16)
        self.item_layout.addStretch(1)
        self.item_layout.addWidget(self.remove_button, 0, Qt.AlignmentFlag.AlignRight)
        self.item_layout.setAlignment(Qt.AlignmentFlag.AlignVCenter)

        self.remove_button.clicked.connect(lambda: self.removed.emit(self))


class IgnoredSettingCard(ExpandSettingCard):
    """Ignored files setting card"""

    ignored_changed = Signal(list)

    def __init__(self, config_item: ConfigItem, icon: str | QIcon | FluentIconBase, title: str, content: str | None = None, parent=None) -> None:  # noqa: ANN001, D417
        """
        Parameters
        ----------
        config_item: ConfigItem
            configuration item operated by the card

        icon: str | QIcon | FluentIconBase
            the icon to be drawn

        content: str
            the content of card

        parent: QWidget
            parent widget
        """

        # Cast parameters to satisfy parent class type requirements
        # FluentIconBase is compatible but not in the parent's type signature
        super().__init__(cast("str | QIcon | Fi", icon), title, cast("str", content), parent)
        self.config_item = config_item
        self.clear_ignored_button = ToolButton(Fi.BROOM, self)
        self.new_ignored_input = LineEdit(self)
        self.add_ignored_button = PushButton(self.tr("Add"), self, Fi.ADD)

        self.ignored = qconfig.get(config_item).copy()  # type:list[str]
        self.ignored_cards = []

        self.__initWidget()

    def __initWidget(self) -> None:
        self.setExpand(True)

        # initialize layout
        self.addWidget(self.clear_ignored_button)
        self.addWidget(self.new_ignored_input)
        self.addWidget(self.add_ignored_button)

        self.viewLayout.setSpacing(0)
        self.viewLayout.setAlignment(Qt.AlignmentFlag.AlignTop)
        self.viewLayout.setContentsMargins(0, 0, 0, 0)
        for i in self.ignored:
            self.__add_ignored_item(i)

        self.clear_ignored_button.setToolTip(self.tr("Clear all"))
        self.clear_ignored_button.installEventFilter(ToolTipFilter(self.clear_ignored_button))
        self.clear_ignored_button.clicked.connect(self.__clear_ignored)
        self.new_ignored_input.setPlaceholderText(self.tr("Ignored"))
        self.add_ignored_button.clicked.connect(self.__add_ignored)

    def __add_ignored(self, name=None) -> None:  # noqa: ANN001
        if not name:
            # Validate input
            new_ignored = self.new_ignored_input.text()
            if len(new_ignored) <= 0:
                self.__show_ignore_failed_tip()
                return
            if new_ignored in self.ignored:
                self.__show_ignore_duplicate_tip()
                return
            name = new_ignored

        self.__add_ignored_item(name)
        self.ignored.append(name)
        self.new_ignored_input.clear()
        qconfig.set(self.config_item, self.ignored)
        self.ignored_changed.emit(self.ignored)

    def __add_ignored_item(self, ignored: str) -> None:
        """add folder item"""
        item = IgnoredItem(ignored, self.view)
        item.removed.connect(lambda: self.__remove_ignored(item))
        self.viewLayout.addWidget(item)
        self.ignored_cards.append(item)
        item.show()
        self._adjustViewSize()

    def ignored_updated(self) -> None:
        ignore_backup = qconfig.get(self.config_item).copy()

        self.__clear_ignored()

        self.ignored = ignore_backup.copy()
        for i in self.ignored:
            self.__add_ignored_item(i)
        qconfig.set(self.config_item, ignore_backup)

    def __remove_ignored(self, item: IgnoredItem) -> None:
        """remove ignored"""
        if item.ignored not in self.ignored:
            return

        self.ignored.remove(item.ignored)
        self.ignored_cards.remove(item)
        self.viewLayout.removeWidget(item)
        item.deleteLater()
        self._adjustViewSize()

        self.ignored_changed.emit(self.ignored)
        qconfig.set(self.config_item, self.ignored)

    def __clear_ignored(self) -> None:
        for card in self.ignored_cards[:]:
            self.__remove_ignored(card)

    def __show_ignore_failed_tip(self) -> None:
        TeachingTip.create(
            target=self.new_ignored_input,
            icon=InfoBarIcon.ERROR,
            title=self.tr("Check your input"),
            content=self.tr("Please enter something"),
            isClosable=True,
            tailPosition=TeachingTipTailPosition.BOTTOM,
            duration=2000,
            parent=self,
        )

    def __show_ignore_duplicate_tip(self) -> None:
        TeachingTip.create(
            target=self.new_ignored_input,
            icon=InfoBarIcon.ERROR,
            title=self.tr("Duplicate"),
            content=self.tr("Your input is a duplicate of an existing ignore"),
            isClosable=True,
            tailPosition=TeachingTipTailPosition.BOTTOM,
            duration=2000,
            parent=self,
        )
