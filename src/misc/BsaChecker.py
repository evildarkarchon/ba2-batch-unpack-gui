"""Module for checking and validating BA2 archive files.

This module provides the BsaChecker class, which runs in a separate thread to scan
directories for BA2 archives and verify their integrity using BSArch.exe.
"""

import pathlib
from typing import Protocol

from PySide6.QtCore import QThread, Signal

from misc.Utilities import extract_ba2, list_ba2, resource_path, scan_for_ba2


class ParentProtocol(Protocol):
    """Protocol defining the interface that BsaChecker expects from its parent."""


class BsaChecker(QThread):
    """Thread for scanning and validating BA2 archive files.

    This class runs in a separate thread to scan a directory for BA2 archives
    and verify their integrity using BSArch.exe. It can perform either a quick
    scan (listing contents) or a deep scan (extracting to temp directory).

    Signals
    -------
    done_processing : Signal[list]
        Emitted when processing is complete with [parent, num_ok, num_failed].
    issue_found : Signal[str]
        Emitted when a corrupted or invalid BA2 archive is found.

    Attributes
    ----------
    path : str
        The directory path to scan for BA2 archives.
    deep_scan : bool
        Whether to perform a deep scan (extraction) or quick scan (list only).
    """

    done_processing = Signal(list)
    issue_found = Signal(str)

    def __init__(self, parent: ParentProtocol, path: str, *, deep_scan: bool = False) -> None:
        """Initialize the BsaChecker thread.

        Parameters
        ----------
        parent : ParentProtocol
            The parent object that created this thread.
        path : str
            The directory path to scan for BA2 archives.
        deep_scan : bool, optional
            If True, performs deep scan with extraction; if False, performs quick
            scan by listing contents only. Default is False.
        """
        super().__init__()
        self._parent = parent
        # self.progress = self._parent.preview_progress
        self.path = path
        self.deep_scan = deep_scan

    def run(self) -> None:
        """Execute the BA2 archive scanning and validation process.

        Scans the configured directory for BA2 archives and validates each one
        using BSArch.exe. Performs either a quick scan (listing contents) or
        deep scan (extraction) based on the deep_scan flag.

        Emits
        -----
        issue_found : Signal[str]
            Emitted for each corrupted or invalid BA2 archive found.
        done_processing : Signal[list]
            Emitted when complete with [parent, num_ok, num_failed].
        """
        ba2_paths = scan_for_ba2(self.path, [".ba2"])
        num_failed = 0
        num_ok = 0

        # self.progress.setMaximum(len(ba2_paths))

        for f in ba2_paths:
            if not self.deep_scan:
                result = list_ba2(f, resource_path("bin/BSArch.exe"))
            else:
                result = extract_ba2(f, resource_path("bin/BSArch.exe"), use_temp=True)
            if result != 0:
                num_failed += 1
                self.issue_found.emit(pathlib.Path(f).resolve())
                # self.progress.error()
            else:
                num_ok += 1
            # self.progress.setValue(self.progress.value() + 1)

        self.done_processing.emit([self._parent, num_ok, num_failed])
