import io

import pytest

import click
import click._termui_impl


@pytest.mark.parametrize("update_min_steps", [1, 2, 3, 7, 20, 25])
@pytest.mark.parametrize("drive", ["iterate", "update"])
def test_the_bar_lands_on_its_total(monkeypatch, update_min_steps, drive):
    monkeypatch.setattr(click._termui_impl, "isatty", lambda _: True)
    stream = io.StringIO()
    if drive == "iterate":
        with click.progressbar(range(20), show_pos=True, update_min_steps=update_min_steps, file=stream) as bar:
            for _ in bar:
                pass
    else:
        with click.progressbar(length=20, show_pos=True, update_min_steps=update_min_steps, file=stream) as bar:
            for _ in range(20):
                bar.update(1)
    assert bar.pos == 20
    assert bar.finished
    assert "20/20" in stream.getvalue()
