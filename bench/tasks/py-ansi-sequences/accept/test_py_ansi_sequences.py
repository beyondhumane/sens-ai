import pytest

import click
from click._compat import term_len


@pytest.mark.parametrize(
    ("text", "expect"),
    [
        ("\x1b[38:2:255:0:0mx y\x1b[0m", "x y"),
        ("\x1b[38:5:200mx y\x1b[0m", "x y"),
        ("\x1b[<0;1;1Mx y\x1b[<0;1;1m", "x y"),
        ("\x1b[0 qx y", "x y"),
        ("x\x1b[3~y", "xy"),
    ],
)
def test_unstyle_removes_any_escape_sequence(text, expect):
    assert click.unstyle(text) == expect


@pytest.mark.parametrize(
    "body",
    [
        "\x1b[38:2:255:0:0malpha\x1b[0m \x1b[38:2:0:255:0mbeta\x1b[0m \x1b[38:2:0:0:255mgamma\x1b[0m \x1b[38:2:9:9:9mdelta\x1b[0m",
        "\x1b[<0;1;1malpha beta gamma delta\x1b[<0;1;1m",
    ],
)
def test_help_wraps_styled_text_on_its_visible_width(body):
    styled = click.formatting.wrap_text(body, width=15)
    plain = click.formatting.wrap_text(click.unstyle(body), width=15)
    assert [click.unstyle(line) for line in styled.splitlines()] == plain.splitlines()


def test_escape_sequences_take_no_width():
    assert term_len("\x1b[38:2:255:0:0mabc\x1b[0m") == 3
    assert term_len("a\x1b[3~b") == 2
