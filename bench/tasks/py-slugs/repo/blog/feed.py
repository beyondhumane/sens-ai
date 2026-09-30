from blog.text import slugify

DOMAIN = "blog.example"


def entry_id(title: str) -> str:
    return f"tag:{DOMAIN},{slugify(title)}"
