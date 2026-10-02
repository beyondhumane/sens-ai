from dataclasses import dataclass

from blog.text import slugify


@dataclass
class Article:
    title: str
    body: str


def url(title: str) -> str:
    return f"/articulos/{slugify(title)}"


def links(articles: list[Article]) -> list[str]:
    return [url(article.title) for article in articles]
