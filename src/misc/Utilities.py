import os
import pathlib
import re
import shlex
import subprocess
import sys
import tempfile
import winreg
from typing import Optional

import requests
from construct import Bytes, Int32ul, Int64ul, PaddedString, StreamError, Struct
from packaging.version import Version
from PySide6.QtWidgets import QApplication

from misc.Config import GITHUB_RELEASE_URL, VERSION, LogLevel, cfg


def is_ignored(file: str) -> bool:
    # File is the full path to the file, so we need to perform a full matching and a partial matching based
    # on the file name
    # Case 1 - Full path matching
    if os.path.abspath(file) in cfg.get(cfg.ignored):
        return True
    # Case 2 - Partial matching
    base_name = os.path.basename(file)
    for ignored in cfg.get(cfg.ignored):
        if "{" in ignored and "}" in ignored:
            # Regex pattern
            pattern = ignored.split("{")[1].split("}")[0]
            if re.fullmatch(pattern, base_name):
                return True
        if ignored in base_name:
            return True
    return False


def resource_path(relative_path: str) -> str:
    """Get absolute path to resource, works for dev and for PyInstaller"""
    if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
        base_path = sys._MEIPASS
    else:
        base_path = os.path.abspath(".")

    return os.path.join(base_path, relative_path)


def get_default_windows_app(suffix: str) -> str:
    try:
        class_root: str = winreg.QueryValue(winreg.HKEY_CLASSES_ROOT, suffix)
        with winreg.OpenKey(winreg.HKEY_CLASSES_ROOT, rf"{class_root}\shell\open\command") as key:
            command: str = winreg.QueryValueEx(key, "")[0]
            return shlex.split(command)[0]
    except FileNotFoundError:
        return ""


header_struct = Struct(
    "magic" / Bytes(4),
    "version" / Int32ul,
    "type" / PaddedString(4, "utf8"),
    "file_count" / Int32ul,
    "names_offset" / Int64ul,
)

units = {"B": 1, "KB": 1000, "MB": 1000**2, "GB": 1000**3, "TB": 1000**4}


def parse_size(size: str) -> int:
    if not (size[-1] == "b" or size[-1] == "B"):
        size = size + "B"
    size = size.upper()
    if not size.startswith(r" "):
        size = re.sub(r"([KMGT]?B)", r" \1", size)
    try:
        parts: list[str] = [string.strip() for string in size.split()]
        number: str
        unit: str
        number, unit = parts
        return int(float(number) * units[unit])
    except ValueError:
        return -1


# Return all ba2 in the folder that contain one of the given postfixes
# Note: it scans for exactly the second-tier directories under the given directory (aka the mod folders)
# This is to avoid scanning for ba2 that will not be loaded to the game anyways
def scan_for_ba2(path: str, postfixes: list[str]) -> list[str]:
    all_ba2: list[str] = []
    for d in os.listdir(path):
        full_path: str = os.path.join(path, d)
        # Skip files
        if not pathlib.Path(full_path).is_dir():
            continue
        # List all files under the mod
        for ba2 in os.listdir(full_path):
            fpath: str = os.path.join(full_path, ba2)
            # Add only *.ba2 archives that contains one of the specified postfixes
            if any([postfix in ba2.lower() for postfix in postfixes]):
                all_ba2.append(fpath)

    return all_ba2


# A convenience function to return the number of files in a ba2 archive
def num_files_in_ba2(file: str) -> int:
    with pathlib.Path(file).open("rb") as fs:
        try:
            result = header_struct.parse_stream(fs)
            return result.file_count
        except StreamError as e:
            QApplication.instance().log_view.add_log(f"Error parsing {file}: {e}", LogLevel.WARNING)
            return -1


def extract_ba2(file: str, bsab_exe_path: str, use_temp: bool = False) -> int:
    tmp_dir: Optional[tempfile.TemporaryDirectory] = None
    extraction_path: str

    if use_temp:
        tmp_dir = tempfile.TemporaryDirectory()
        extraction_path = tmp_dir.name
    else:
        cfg_path: str = cfg.get(cfg.extraction_path)
        if cfg_path:
            if pathlib.Path(cfg_path).is_absolute():
                extraction_path = cfg_path
            else:
                extraction_path = os.path.join(os.path.dirname(file), cfg_path)
        else:
            extraction_path = os.path.dirname(file)

        if not pathlib.Path(extraction_path).is_dir():
            pathlib.Path(extraction_path).mkdir(parents=True)

    # Hide the console window
    si: subprocess.STARTUPINFO = subprocess.STARTUPINFO()
    si.dwFlags |= subprocess.STARTF_USESHOWWINDOW

    args: list[str] = [
        bsab_exe_path,
        "unpack",
        file,
        extraction_path,
    ]
    proc: subprocess.CompletedProcess[str] = subprocess.run(args, check=False, text=True, capture_output=True, startupinfo=si)

    if use_temp:
        tmp_dir.cleanup()
    results: list[str] = proc.stdout.strip().split("\n")
    if "Error:" in results[-1] or "error:" in results[-1]:
        QApplication.instance().log_view.add_log(f"Error reading {file}", LogLevel.WARNING)
        return -1
    return 0
    # if proc.returncode != 0:
    #     QApplication.instance().log_view.add_log(f'Error extracting {file}', LogLevel.WARNING)
    #     return -1
    # else:
    #     QApplication.instance().log_view.add_log(f'{proc.stdout}', LogLevel.INFO)
    #     return 0


def list_ba2(file: str, bsab_exe_path: str) -> int:
    # Hide the console window
    si: subprocess.STARTUPINFO = subprocess.STARTUPINFO()
    si.dwFlags |= subprocess.STARTF_USESHOWWINDOW

    args: list[str] = [
        bsab_exe_path,
        file,
        "-list",
    ]
    proc: subprocess.CompletedProcess[str] = subprocess.run(args, check=False, text=True, capture_output=True, startupinfo=si)
    results: list[str] = proc.stdout.strip().split("\n")
    if "Error:" in results[-1] or "error:" in results[-1]:
        QApplication.instance().log_view.add_log(f"Error reading {file}", LogLevel.WARNING)
        return -1
    return 0

    # # BSArch does not return the status code correctly, we have to check for the error message
    # if proc.returncode != 0:
    #     QApplication.instance().log_view.add_log(f'Error reading {file}', LogLevel.WARNING)
    #     return -1
    # else:
    #     return 0


def check_latest_version() -> Optional[str]:
    r: requests.Response = requests.get(GITHUB_RELEASE_URL)
    version: str = r.url.split("/")[-1]
    if Version(version) > Version(VERSION):
        return version
    return None
