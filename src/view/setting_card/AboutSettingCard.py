from misc.Config import AUTHOR, GITHUB_URL, NEXUS_URL, SWC_URL, VERSION, YEAR
from PySide6.QtCore import QUrl
from PySide6.QtWidgets import QHBoxLayout, QWidget
from qfluentwidgets import BodyLabel, ExpandGroupSettingCard, HyperlinkLabel, PrimaryPushButton
from qfluentwidgets import FluentIcon as Fi


class AboutSettingCard(ExpandGroupSettingCard):
    def __init__(self, parent=None):
        super().__init__(
            Fi.INFO,
            self.tr("About"),
            "© " + self.tr("Copyright") + f" {YEAR}, {AUTHOR}. " + self.tr("Version") + f" {VERSION}",
            parent,
        )

        # Feedback
        self.feedback_label = BodyLabel(self.tr("Provide feedback"), self)
        self.feedback_button = PrimaryPushButton(self.tr("Feedback"), self)

        # Nexus
        self.nexus_label = BodyLabel(self.tr("Check manual/information"), self)
        self.nexus_link = HyperlinkLabel(QUrl(NEXUS_URL), self.tr("Nexus page"), self)
        # self.nexus_link.setText(self.tr('Nexus page'))

        # GitHub
        self.github_label = BodyLabel(self.tr("Unpackrr on GitHub"), self)
        self.github_link = HyperlinkLabel(QUrl(GITHUB_URL), self.tr("Source code"), self)
        # self.github_link.setText(self.tr('Source code'))

        # SWC
        self.swc_label = BodyLabel(self.tr("Organically & locally produced by Southwest Codeworks\nMade with ❤️ in Arizona"), self)
        self.swc_link = HyperlinkLabel(QUrl(SWC_URL), self.tr("Project page"), self)
        # self.swc_link.setText(self.tr('Project page'))

        self.__add(self.feedback_label, self.feedback_button)
        self.__add(self.nexus_label, self.nexus_link)
        self.__add(self.github_label, self.github_link)
        self.__add(self.swc_label, self.swc_link)

    def __add(self, label, widget):
        w = QWidget()
        w.setFixedHeight(60)

        layout = QHBoxLayout(w)
        layout.setContentsMargins(48, 12, 48, 12)

        layout.addWidget(label)
        layout.addStretch(1)
        layout.addWidget(widget)

        # Add the widget group to the setting card
        self.addGroupWidget(w)
