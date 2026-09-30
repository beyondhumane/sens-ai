import unittest

from blog import articles, feed, tags
from blog.text import slugify


class AccentsTest(unittest.TestCase):
    def test_article_urls_keep_accented_letters_as_plain_letters(self):
        self.assertEqual(articles.url("Canción de otoño"), "/articulos/cancion-de-otono")

    def test_tags_and_feed_entries_get_the_same_fix(self):
        self.assertEqual(tags.slug_for("Año Nuevo"), "ano-nuevo")
        self.assertEqual(tags.cloud(["Café", "cafe"]), {"cafe": 2})
        self.assertEqual(feed.entry_id("Crème brûlée"), "tag:blog.example,creme-brulee")

    def test_plain_titles_still_work(self):
        self.assertEqual(slugify("Hello, World!"), "hello-world")

    def test_titles_that_are_too_long_are_still_refused(self):
        with self.assertRaises(ValueError):
            slugify("é" * 201)


if __name__ == "__main__":
    unittest.main()
