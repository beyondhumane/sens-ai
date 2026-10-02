import unittest

from blog import articles, feed, tags
from blog.text import MAX_SLUG, slugify


class SlugifyTest(unittest.TestCase):
    def test_plain_titles_become_lowercase_words_joined_by_hyphens(self):
        self.assertEqual(slugify("Hello, World!"), "hello-world")

    def test_slugs_are_capped_without_a_trailing_hyphen(self):
        slug = slugify("word " * 40)
        self.assertLessEqual(len(slug), MAX_SLUG)
        self.assertFalse(slug.endswith("-"))

    def test_titles_that_are_too_long_are_refused(self):
        with self.assertRaises(ValueError):
            slugify("x" * 201)

    def test_every_caller_uses_the_same_slug(self):
        self.assertEqual(articles.url("Hello World"), "/articulos/hello-world")
        self.assertEqual(tags.cloud(["Rust", "rust", "Go"]), {"rust": 2, "go": 1})
        self.assertEqual(feed.entry_id("Hello World"), "tag:blog.example,hello-world")


if __name__ == "__main__":
    unittest.main()
