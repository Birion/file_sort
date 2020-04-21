#! /usr/bin/env python3

import json
import re
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Tuple, Union

try:
    from specials import special
except ImportError:
    special = {}

replace_pattern = re.compile(r".*<(.*)>.*")

_config = Path(__file__).parent / "config.json"

MapType = Dict[str, Union[str, List[str]]]
ComicFunction = Callable[[str], str]
FunctionTuple = Tuple[str, ComicFunction]


@dataclass
class Mapping:
    title: str
    dir: Path
    original: str
    new: str
    fn: Optional[FunctionTuple]

    @classmethod
    def from_config(cls, config: Dict[str, str], root: str):
        title = config["title"]

        root = Path(root)

        try:
            dir = root.joinpath(config["directory"])
        except TypeError:
            dir = root.joinpath(*config["directory"])

        if replace_pattern.match(config["pattern"]):
            original_pattern = re.sub(r"[<>]", "", config["pattern"])
            new_pattern = replace_pattern.match(config["pattern"]).group(1)
        else:
            original_pattern = config["pattern"]
            new_pattern = config["pattern"]
        try:
            fn = special[config["function"]]
        except KeyError:
            fn = None

        return cls(title, dir, original_pattern, new_pattern, fn)


@dataclass
class FSManager:
    root: Path
    download: Path
    files: List[Path]

    @staticmethod
    def parse(src: List[str]) -> Path:
        root = src[0] if not src[0].endswith(":") else f"{src[0]}/"
        return Path(root).joinpath(*src[1:]).expanduser()

    @classmethod
    def from_config(cls, config: Dict[str, List[Union[str, MapType]]]):
        root = cls.parse(config["root"])
        download = cls.parse(config["download"])
        files = [x for x in download.iterdir() if x.is_file()]
        return cls(root, download, files)


@dataclass
class Processor:
    _file: Path
    _target: Path = Path()

    def parse_dir(self, directory: Path) -> Path:
        replacement = replace_pattern.match(str(directory))
        if not replacement:
            return directory
        else:
            start, length = map(int, replacement.group(1).split(":"))
            replace_part = self.file[start : start + length]
            new_dir = re.sub(
                r"<" + replacement.group(1) + r">", replace_part, str(directory)
            )
            return Path(new_dir)

    def parse_file(self, pattern: str) -> str:
        replacement = re.search(pattern, self.file)
        return replacement.group()

    def make_target_dir(self, dir: Path):
        self._target = self.parse_dir(dir)
        try:
            self._target.mkdir(parents=True)
            print(f"{self._target} doesn't exist. Creating directory.")
        except FileExistsError:
            pass

    def make_dst(self, new_name: str, _fn: Optional[FunctionTuple]) -> Path:
        dst = self.parse_file(new_name)
        if _fn:
            arg, fn = _fn
            if arg == "dir":
                dst = fn(self.target)
            elif arg == "filename":
                dst = fn(self.file)

        return self._target.joinpath(dst)

    @property
    def file(self):
        return self._file.name

    @property
    def target(self):
        return str(self._target)


def process_file(file: Path, mappings: List[Mapping]):
    processor = Processor(file)

    for mapping in mappings:
        if re.match(mapping.original, processor.file):
            print(f"{file} found! Applying setup for {mapping.title}")

            processor.make_target_dir(mapping.dir)

            source = fs.download.joinpath(processor.file)
            target = processor.make_dst(mapping.new, mapping.fn)

            shutil.move(Path(source), Path(target))


def run() -> None:
    with _config.open(encoding="utf-8") as fp:
        config = json.load(fp)

    fs = FSManager.from_config(config)

    mappings = [
        Mapping.from_config(mapping, str(fs.root)) for mapping in config["mappings"]
    ]

    for file in fs.files:
        process_file(file, mappings)


if __name__ == "__main__":
    run()
