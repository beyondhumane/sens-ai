import re

MAX_TITLE = 200
MAX_SLUG = 60


def slugify(title: str) -> str:
    if len(title) > MAX_TITLE:
        raise ValueError(f"title longer than {MAX_TITLE} characters")
    slug = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
    return slug[:MAX_SLUG].rstrip("-")
