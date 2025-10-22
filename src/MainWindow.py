import ctypes
import pathlib
import sys
from typing import TYPE_CHECKING, Any

from PySide6.QtCore import QLocale, QRect, QTranslator, Signal
from PySide6.QtGui import QIcon
from PySide6.QtWidgets import QApplication
from qfluentwidgets import FluentIcon as Fi
from qfluentwidgets import FluentTranslator, NavigationItemPosition, SplashScreen, SplitFluentWindow, Theme

from misc.Config import cfg
from misc.Utilities import resource_path
from prefab.CustomIcon import CustomIcon
from view.CheckFileScreen import CheckFileScreen
from view.LogView import LogLevel, LogView
from view.MainScreen import MainScreen
from view.SettingScreen import SettingScreen

if TYPE_CHECKING:
    from collections.abc import Callable

basedir: str = str(pathlib.Path(__file__).parent)


class MainWindow(SplitFluentWindow):
    def __init__(self) -> None:
        super().__init__()

        self.splash: SplashScreen = SplashScreen(QIcon(resource_path("resources/images/unpackrr.png")), self)
        self.init_window()
        self.show()

        # Create split tabs
        self.main_screen: MainScreen = MainScreen(self)
        self.check_file_screen: CheckFileScreen = CheckFileScreen(self)
        self.setting_screen: SettingScreen = SettingScreen(self)

        self.init_navigation()

        self.splash.finish()

        # Fix the weird InfoBar sizing issue
        self.resize(1001, 700)
        self.resize(1000, 700)

    def init_navigation(self) -> None:
        self.navigationInterface.setReturnButtonVisible(False)
        self.addSubInterface(self.main_screen, Fi.HOME, self.tr("Extraction"))
        self.addSubInterface(self.check_file_screen, CustomIcon.STETHOSCOPE, self.tr("Check Files"))
        self.addSubInterface(self.setting_screen, Fi.SETTING, self.tr("Settings"), NavigationItemPosition.BOTTOM)

    def init_window(self) -> None:
        self.setWindowTitle("Unpackrr")
        self.setWindowIcon(QIcon(resource_path("resources/images/unpackrr.png")))
        # Center window
        desktop: QRect = QApplication.screens()[0].availableGeometry()
        w: int = desktop.width()
        h: int = desktop.height()
        self.move(w // 2 - self.width(), h // 2 - self.height() // 2)
        # Set size
        self.setMinimumSize(800, 500)
        self.resize(1000, 700)


# Hack to install a "global" signal/slot  # noqa: FIX004
class Unpackrr(QApplication):
    ignore_changed = Signal()
    theme_changed = Signal()

    def __init__(self, argv: list[str]) -> None:
        super().__init__(argv)
        self.log_view: LogView = LogView()
        self.log_view.resize(640, 480)
        self.log_view.setWindowTitle("Unpackrr Logs")
        self.log_view.setWindowIcon(QIcon(resource_path("resources/images/unpackrr.png")))

        self.log_view.add_log("Unpackrr started", LogLevel.INFO)

        self.old_hook: Callable[..., Any] = sys.excepthook
        sys.excepthook = self.log_view.catch_exception


if __name__ == "__main__":
    app: Unpackrr = Unpackrr(sys.argv)

    # internationalization
    locale: QLocale = cfg.get(cfg.language).value
    fluent_translator: FluentTranslator = FluentTranslator(locale)
    app.installTranslator(fluent_translator)

    if locale.language().name != "English":
        unpackrr_translator: QTranslator = QTranslator()
        unpackrr_translator.load(locale, "unpackrr", ".", resource_path("resources/i18n"))
        app.installTranslator(unpackrr_translator)

    if cfg.get(cfg.show_debug):
        app.log_view.show()

    # Required to display icons correctly
    app_id: str = "unpackrr"
    ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID(app_id)

    # Set the default theme to Auto
    if cfg.get(cfg.first_launch):
        cfg.set(cfg.themeMode, Theme.AUTO)

    w: MainWindow = MainWindow()
    ret: int = app.exec()

    cfg.set(cfg.first_launch, False)

    sys.exit(ret)
