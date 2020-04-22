import re
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

import pendulum

ComicFunction = Callable[[str], str]


@dataclass
class ProcessingFunction:
    arg: str
    fn: ComicFunction


def bloomin_faeries(folder: str) -> str:
    counter_pattern = re.compile(r"(?P<date>20\d{2}-(0[1-9]|1[0-2])-([0-2][1-9]|3[01]))-BF(?P<counter>\d+)_Heather.*")
    current = Path(folder).glob("*")
    latest_comic = list(current)[-1].name
    latest_comic_regex = counter_pattern.match(latest_comic)
    counter = int(latest_comic_regex.group("counter"))
    today = pendulum.today().to_date_string()
    counter += 1
    counter = str(counter).zfill(4)
    return f"{today}-BF{counter}_Heather.jpg"



special = {
    "bloomin_faeries": ProcessingFunction("dir", bloomin_faeries),
}
