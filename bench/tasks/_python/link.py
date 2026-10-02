import pathlib
import site
import tomllib

project = tomllib.loads(pathlib.Path("pyproject.toml").read_text(encoding="utf-8"))["project"]
packages = pathlib.Path(site.getsitepackages()[-1])
(packages / "work.pth").write_text(str(pathlib.Path("src").resolve()) + "\n", encoding="utf-8")
metadata = packages / f"{project['name']}-{project['version']}.dist-info"
metadata.mkdir(exist_ok=True)
(metadata / "METADATA").write_text(f"Metadata-Version: 2.1\nName: {project['name']}\nVersion: {project['version']}\n", encoding="utf-8")
