from traceback import format_exception
from types import TracebackType

from PySide6 import QtCore
from PySide6.QtGui import QCloseEvent, QColor
from PySide6.QtWidgets import QWidget
from qfluentwidgets import TextEdit

from misc.Config import LogLevel, cfg


# Adopted from a code snippet by eyllanesc from StackOverflow. Original post at https://stackoverflow.com/a/63853259
class LogView(TextEdit):
    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setReadOnly(True)
        self.setFontPointSize(10)
        self._process: QtCore.QProcess = QtCore.QProcess()
        self._process.readyReadStandardOutput.connect(self.handle_stdout)
        self._process.readyReadStandardError.connect(self.handle_stderr)

    def start_log(self, program: str, arguments: list[str] | None = None) -> None:
        if arguments is None:
            arguments = []
        self._process.start(program, arguments)

    def add_log(self, message: str, level: LogLevel = LogLevel.INFO) -> None:
        # The higher the level the more trivial the message is
        # (perhaps the opposite of what you would expect)
        if level.value > cfg.get(cfg.log_level).value:
            return
        color: QColor
        if level in {LogLevel.FATAL, LogLevel.ERROR}:
            color = QColor("red")
        elif level == LogLevel.WARNING:
            # Light red
            color = QColor(255, 106, 91)
        elif level == LogLevel.INFO:
            color = QColor("white")
        else:
            color = QColor("gray")
        self.setTextColor(color)
        self.append(f"{level.name}: {message.rstrip()}")

    def handle_stdout(self) -> None:
        data = self._process.readAllStandardOutput()
        message: str = data.toStdString()
        self.add_log(message)

    def handle_stderr(self) -> None:
        data = self._process.readAllStandardError()
        message: str = data.toStdString()
        self.add_log(message)

    # Capture Python-originated exceptions
    def catch_exception(self, _type: type[BaseException], value: BaseException, _traceback: TracebackType | None) -> None:
        traceback_str: str = "".join(format_exception(_type, value, _traceback))
        self.add_log(traceback_str, LogLevel.ERROR)

    # Revert the show_debug setting to False when the window is closed
    def closeEvent(self, event: QCloseEvent) -> None:
        super().closeEvent(event)
        cfg.set(cfg.show_debug, False)
