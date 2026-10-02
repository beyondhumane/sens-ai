import pytest

import click


@pytest.mark.parametrize(
    ("call", "user_input", "color", "expect"),
    [
        (lambda: click.confirm(click.style("Hello World!", fg="green")), "y", False, "Hello World! [y/N]: y\n"),
        (lambda: click.confirm(click.style("Hello World!", fg="green")), "y", True, "\x1b[32mHello World!\x1b[0m [y/N]: y\n"),
        (lambda: click.prompt(click.style("Name", fg="green")), "Bob", False, "Name: Bob\n"),
        (lambda: click.prompt(click.style("Name", fg="green")), "Bob", True, "\x1b[32mName\x1b[0m: Bob\n"),
    ],
)
def test_prompts_keep_or_strip_colors_like_echo(runner, call, user_input, color, expect):
    @click.command()
    def cli():
        call()

    result = runner.invoke(cli, input=user_input, color=color)
    assert result.output == expect
