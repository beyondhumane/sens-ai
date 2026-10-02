from blog.text import slugify


def slug_for(tag: str) -> str:
    return slugify(tag)


def cloud(tags: list[str]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for tag in tags:
        key = slug_for(tag)
        counts[key] = counts.get(key, 0) + 1
    return counts
