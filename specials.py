import re
from pathlib import Path

import pendulum


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


def yuuki(filename: str) -> str:
    return re.sub(r"i(\d)-p(\d)(-xxx)\.", r"i0\1-p0\2\3.", filename)


def egs_sketch(filename: str) -> str:
    today = pendulum.today().format("YYYYMMDD")
    keep_part = re.search(r"sketch-\d+.png", filename)
    return f"fgs{today}-{keep_part.group()}"


def egs_np(filename: str) -> str:
    pattern = re.search(
        r"\d+-(np)(20[1-4][0-9])((0)3([1-9])|(1)3([12]))([0-2][0-9]|3[01])(-.+\.png)",
        filename,
    )
    pattern = pattern.groups()
    prefix = pattern[0]
    year = pattern[1]
    if pattern[3]:
        month = "".join(pattern[3:5])
    else:
        month = "".join(pattern[5:7])
    day = pattern[-2]
    suffix = pattern[-1]
    new_filename = "".join((prefix, year, month, day, suffix))
    return new_filename


def egs(filename: str) -> str:
    stamp, suffix = filename.split("-1904-")
    creation_date = pendulum.from_timestamp(int(stamp)).format("YYYYMMDD")
    return "-".join((creation_date, suffix))


def hell_high(filename: str) -> str:
    stamp, suffix = filename.split("-")
    creation_date = pendulum.from_timestamp(int(stamp)).format("YYYYMMDD")
    return "-".join((creation_date, suffix))


special = {
    "bloomin_faeries": ("dir", bloomin_faeries),
    "yuuki": ("filename", yuuki),
    "egs_sketch": ("filename", egs_sketch),
    "egs_np": ("filename", egs_np),
    "egs": ("filename", egs),
    "hell_high": ("filename", hell_high),
}
