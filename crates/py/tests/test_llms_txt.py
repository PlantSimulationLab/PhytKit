"""docs/hooks/llms_txt.py: the llms.txt / llms-full.txt the docs build writes, run on the real
docs tree without MkDocs, so they can't go stale or point at missing pages."""
from __future__ import annotations

import importlib.util
import re
from pathlib import Path

import pytest

yaml = pytest.importorskip("yaml")  # PyYAML comes with mkdocs; CI installs it for these tests

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs" / "docs"
_spec = importlib.util.spec_from_file_location("llms_txt", ROOT / "docs" / "hooks" / "llms_txt.py")
llms = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(llms)


class _Loader(yaml.SafeLoader):
    pass


# mkdocs.yml may use !!python/name tags for extensions; the nav needs none of them.
_Loader.add_multi_constructor("tag:yaml.org,2002:python/", lambda loader, suffix, node: None)


def _config():
    cfg = yaml.load((ROOT / "docs" / "mkdocs.yml").read_text(encoding="utf-8"), Loader=_Loader)
    assert "hooks/llms_txt.py" in cfg["hooks"], "the hook is not registered in mkdocs.yml"
    return cfg


def _render():
    cfg = _config()
    args = (cfg["nav"], DOCS, cfg["site_url"], cfg["site_name"], cfg["site_description"])
    return cfg, llms.llms_txt(*args), llms.llms_full_txt(*args, ROOT)


def test_llms_txt_links_every_user_page_and_only_existing_pages():
    cfg, index, _ = _render()
    assert index.startswith(f"# {cfg['site_name']}\n\n> ")
    pages = [p for _, p in llms.flatten(cfg["nav"])]
    for p in pages:
        assert llms.url_for(cfg["site_url"], p) in index, p
    for url in re.findall(r"\]\((https://[^)]+)\)", index):
        rel = url[len(cfg["site_url"]):].rstrip("/")
        assert rel == "" or (DOCS / f"{rel}.md").is_file() or (DOCS / rel / "index.md").is_file(), url
    assert "## Optional" in index
    assert "pk describe" in index and "pk agent-files" in index


def test_llms_full_txt_holds_the_pages_and_the_skill():
    _, _, full = _render()
    for page in ("guide/agents.md", "guide/python.md", "reference/output.md", "workflows/fit-aci-curve.md"):
        assert (DOCS / page).read_text(encoding="utf-8").strip() in full, page
    skill = ROOT / "crates" / "py" / "python" / "phytkit" / "skills" / "phytkit"
    for f in [skill / "SKILL.md", *(skill / "references").glob("*.md")]:
        assert f.read_text(encoding="utf-8").strip() in full, f.name
    assert "Source: https://" in full
    # Developer pages are linked, not included.
    assert (DOCS / "developers" / "building.md").read_text(encoding="utf-8").strip() not in full


def test_urls_follow_directory_urls():
    assert llms.url_for("https://e.org/P/", "guide/index.md") == "https://e.org/P/guide/"
    assert llms.url_for("https://e.org/P/", "guide/install.md") == "https://e.org/P/guide/install/"
    assert llms.url_for("https://e.org/P/", "index.md") == "https://e.org/P/"
