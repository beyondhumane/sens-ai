import click


def fruit():
    @click.command()
    @click.option("--fruit", type=click.Choice(["apple", "orange", "banana"]))
    def cli(fruit):
        click.echo(fruit)

    return cli


def test_a_close_value_gets_a_suggestion(runner):
    result = runner.invoke(fruit(), ["--fruit", "ornge"])
    assert result.exit_code == 2
    assert "Did you mean 'orange'?" in result.output


def test_a_value_close_to_nothing_gets_none(runner):
    result = runner.invoke(fruit(), ["--fruit", "zzz"])
    assert result.exit_code == 2
    assert "Did you mean" not in result.output
    assert "'zzz' is not one of 'apple', 'orange', 'banana'." in result.output


def test_a_valid_choice_still_passes(runner):
    assert runner.invoke(fruit(), ["--fruit", "banana"]).output == "banana\n"
