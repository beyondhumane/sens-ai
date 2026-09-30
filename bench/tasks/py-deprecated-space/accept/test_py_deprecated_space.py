import pytest

import click


@pytest.mark.parametrize(("deprecated", "expected"), [(True, "(DEPRECATED)"), ("USE B INSTEAD", "(DEPRECATED: USE B INSTEAD)")])
@pytest.mark.parametrize("help_text", ["", None])
def test_an_option_without_help_shows_only_the_label(help_text, deprecated, expected):
    option = click.Option(["--foo"], help=help_text, deprecated=deprecated)
    context = click.Context(click.Command("cli"))
    assert option.get_help_record(context)[1] == expected


@pytest.mark.parametrize("deprecated", [True, "USE OTHER COMMAND INSTEAD"])
@pytest.mark.parametrize("doc", ["", None])
def test_a_command_without_help_shows_only_the_label(runner, doc, deprecated):
    @click.command(deprecated=deprecated, help=doc)
    def cli():
        pass

    shown = runner.invoke(cli, ["--help"]).output
    assert "\n  (DEPRECATED" in shown
    assert "\n   (DEPRECATED" not in shown
