import os

import pytest

import click


@pytest.fixture
def home(tmp_path, monkeypatch):
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("USERPROFILE", str(tmp_path))
    (tmp_path / "notes.txt").write_text("hola", encoding="utf-8")
    (tmp_path / "folder").mkdir()
    return tmp_path


def test_the_home_folder_is_expanded_when_the_path_is_resolved(home):
    found = click.Path(exists=True, resolve_path=True).convert("~/notes.txt", None, None)
    assert found == os.path.realpath(home / "notes.txt")


def test_every_check_still_applies_to_an_expanded_path(home):
    with pytest.raises(click.BadParameter, match="is a directory"):
        click.Path(exists=True, resolve_path=True, dir_okay=False).convert("~/folder", None, None)
    with pytest.raises(click.BadParameter, match="is a file"):
        click.Path(exists=True, resolve_path=True, file_okay=False).convert("~/notes.txt", None, None)
    with pytest.raises(click.BadParameter, match="does not exist"):
        click.Path(exists=True, resolve_path=True).convert("~/missing.txt", None, None)


def test_without_resolving_the_path_is_left_as_given(home):
    assert click.Path().convert("~/notes.txt", None, None) == "~/notes.txt"
