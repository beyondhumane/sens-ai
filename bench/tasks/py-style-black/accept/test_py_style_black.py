import pytest

import click


def test_black_is_a_color_like_any_other():
    assert click.style("x", fg=0) == "\x1b[38;5;0mx\x1b[0m"
    assert click.style("x", bg=0) == "\x1b[48;5;0mx\x1b[0m"
    assert click.style("x", fg=1) == "\x1b[38;5;1mx\x1b[0m"


def test_no_color_stays_no_color():
    assert click.style("x") == "x\x1b[0m"
    assert click.style("x", fg=None, bg=None) == "x\x1b[0m"


def test_an_unknown_color_is_still_refused():
    with pytest.raises(TypeError):
        click.style("x", fg="not-a-color")
    with pytest.raises(TypeError):
        click.style("x", bg="not-a-color")
