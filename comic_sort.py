#! /usr/bin/env python3

import json
import re
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Dict, List, Optional, Union

import pendulum

replace_pattern = re.compile(r".*<(.*)>.*")

_config = Path(__file__).parent / "config.json"

Replacement = Dict[str, str]
FolderPath = List[str]
ConfigItem = Union[str, FolderPath, Dict[str, Replacement]]
ProcessorItem = Dict[str, Union[str, Replacement]]

MapType = Dict[str, ConfigItem]
Config = Dict[str, Union[str, Dict[str, str]]]


@dataclass
class Parser:
    splitter: Optional[str]
    merger: Optional[str]
    format: Optional[str]
    pattern: Optional[re.Pattern]
    replacement: Optional[str]

    @classmethod
    def from_config(cls, processors: ProcessorItem):
        pattern = re.compile(processors.get("pattern")) if "pattern" in processors.keys() else None
        replacement = processors["replacement"]["slash"] if "replacement" in processors.keys() else None
        return cls(
            processors.get("splitter"),
            processors.get("merger"),
            processors.get("format"),
            pattern,
            replacement,
        )


@dataclass
class Mapping:
    title: str
    dir: Path
    original: str
    new: str
    parser: Optional[Parser]

    @classmethod
    def from_config(cls, config: Config, root: str):
        title = config["title"]

        root = Path(root)

        try:
            folder = root.joinpath(config["directory"])
        except TypeError:
            folder = root.joinpath(*config["directory"])

        if replace_pattern.match(config["pattern"]):
            original_pattern = re.sub(r"[<>]", "", config["pattern"])
            new_pattern = replace_pattern.match(config["pattern"]).group(1)
        else:
            original_pattern = config["pattern"]
            new_pattern = config["pattern"]
        processors = config.get("processors")
        parser = Parser.from_config(processors) if processors else None

        return cls(
            title,
            folder,
            original_pattern,
            new_pattern,
            parser
        )


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
        start, length = map(int, replacement.group(1).split(":"))
        replace_part = self.file[start: start + length]
        new_dir = re.sub(
            r"<" + replacement.group(1) + r">", replace_part, str(directory)
        )
        return Path(new_dir)

    def parse_file(self, pattern: str) -> str:
        replacement = re.search(pattern, self.file)
        return replacement.group()

    def make_target_dir(self, folder: Path):
        self._target = self.parse_dir(folder)
        try:
            self._target.mkdir(parents=True)
            print(f"{self._target} doesn't exist. Creating directory.")
        except FileExistsError:
            pass

    def make_dst(self, new_name: str, mapping: Mapping) -> Path:
        dst = self.parse_file(new_name)
        # if mapping.fn:
        #     if mapping.fn.arg == "dir":
        #         dst = mapping.fn.fn(self.target)
        #     elif mapping.fn.arg == "filename":
        #         dst = mapping.fn.fn(self.file)
        if mapping.parser:
            if mapping.parser.splitter:
                stamp, suffix = dst.split(mapping.parser.splitter)
                fmt = mapping.parser.format.format(year="YYYY", month="MM", day="DD")
                creation_date = pendulum.from_timestamp(int(stamp)).format(fmt)
                merger = mapping.parser.merger if mapping.parser.merger else "-"
                dst = merger.join((creation_date, suffix))
            if mapping.parser.pattern:
                dst = mapping.parser.pattern.sub(mapping.parser.replacement, dst)

        return self._target.joinpath(dst)

    @property
    def file(self):
        return self._file.name

    @property
    def target(self):
        return str(self._target)


def process_file(file: Path, mappings: List[Mapping], fs: FSManager):
    processor = Processor(file)

    for mapping in mappings:
        if re.match(mapping.original, processor.file):

            processor.make_target_dir(mapping.dir)

            source = fs.download.joinpath(processor.file)
            target = processor.make_dst(mapping.new, mapping)
            print(f"{processor.file} found! Applying setup for {mapping.title}.")
            if target.name != processor.file:
                print(f"New filename: {target.name}")
            print()

            shutil.move(Path(source), Path(target))


def run() -> None:
    with _config.open(encoding="utf-8") as fp:
        config = json.load(fp)

    fs = FSManager.from_config(config)

    mappings = [
        Mapping.from_config(mapping, str(fs.root)) for mapping in config["mappings"]
    ]

    for file in fs.files:
        process_file(file, mappings, fs)


if __name__ == "__main__":
    run()
