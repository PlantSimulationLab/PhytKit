"""`pk agent-files` and `pk skills`: the same marker-line and opt-out rules as Helios and
Phytograph, generated from the shipped skill so they can't drift from it."""
import json
import re
import subprocess
import sys
from pathlib import Path

import pytest

import phytkit as pk

MARKER = "<!-- phytkit-managed-file"
SKILL = pk.skills_dir() / "phytkit" / "SKILL.md"


def pk_cli(*args, env=None, cwd=None):
    """Run the package's pk (python -m phytkit) and return (exit code, parsed or raw output)."""
    import os

    e = dict(os.environ)
    e.pop("PHYTKIT_AGENT_FILES", None)
    e.update(env or {})
    p = subprocess.run([sys.executable, "-m", "phytkit", *args], capture_output=True, text=True, env=e, cwd=cwd)
    try:
        return p.returncode, json.loads(p.stdout)
    except json.JSONDecodeError:
        return p.returncode, p.stdout


def statuses(res):
    return {f["name"]: f["status"] for f in res["files"]}


def test_writes_both_files_from_the_skill(tmp_path):
    res = pk.agent_files(tmp_path)
    assert statuses(res) == {"AGENTS.md": "written", "CLAUDE.md": "written"}
    agents = (tmp_path / "AGENTS.md").read_text(encoding="utf-8")
    claude = (tmp_path / "CLAUDE.md").read_text(encoding="utf-8")
    first = agents.splitlines()[0]
    assert first.startswith(MARKER) and "Delete this line" in first
    assert claude == first + "\n\n@AGENTS.md\n"
    assert f"`phytkit` {pk.__version__}" in agents
    assert SKILL.resolve().as_posix() in agents
    # The whole skill body is there, minus its frontmatter and its own H1.
    body = SKILL.read_text(encoding="utf-8").split("\n---\n", 1)[1]
    for line in body.splitlines():
        if line.startswith("# ") or "](references/" in line or not line.strip():
            continue
        assert line in agents, line
    assert len(re.findall(r"^# ", agents, re.M)) == 1
    assert "](references/" not in agents
    for target in re.findall(r"\]\((/[^)]+\.md)\)", agents):
        assert Path(target).is_file(), target


def test_second_run_is_unchanged_and_stale_files_are_refreshed(tmp_path):
    pk.agent_files(tmp_path)
    assert statuses(pk.agent_files(tmp_path)) == {"AGENTS.md": "unchanged", "CLAUDE.md": "unchanged"}
    stale = tmp_path / "AGENTS.md"
    stale.write_text(stale.read_text().splitlines()[0] + "\nold text\n")
    assert statuses(pk.agent_files(tmp_path))["AGENTS.md"] == "written"
    assert "old text" not in stale.read_text()


def test_a_file_without_the_marker_on_line_one_is_left_alone(tmp_path):
    mine = "# My notes\n" + MARKER + " (not on the first line) -->\n"
    (tmp_path / "AGENTS.md").write_text(mine)
    res = pk.agent_files(tmp_path)
    assert statuses(res)["AGENTS.md"] == "user-owned"
    assert (tmp_path / "AGENTS.md").read_text() == mine
    assert any("--stdout" in n for n in res["notes"])


def test_users_own_claude_md_gets_a_note_until_it_imports_agents_md(tmp_path):
    (tmp_path / "CLAUDE.md").write_text("# mine\n")
    res = pk.agent_files(tmp_path)
    assert statuses(res)["CLAUDE.md"] == "user-owned"
    assert any("@AGENTS.md" in n for n in res["notes"])
    (tmp_path / "CLAUDE.md").write_text("# mine\n@AGENTS.md\n")
    assert pk.agent_files(tmp_path)["notes"] == []


@pytest.mark.parametrize("value", ["0", "off", "false", "no", ""])
def test_opt_out_writes_nothing(tmp_path, value):
    code, res = pk_cli("agent-files", str(tmp_path), env={"PHYTKIT_AGENT_FILES": value})
    assert code == 0 and res["disabled"] is True
    assert list(tmp_path.iterdir()) == []


def test_stdout_and_dry_run_write_nothing(tmp_path):
    code, text = pk_cli("agent-files", "--stdout", env={"PHYTKIT_AGENT_FILES": "0"})
    assert code == 0 and text.startswith(MARKER)
    res = pk.agent_files(tmp_path, dry_run=True)
    assert statuses(res) == {"AGENTS.md": "would be written", "CLAUDE.md": "would be written"}
    assert list(tmp_path.iterdir()) == []


def test_missing_folder_is_an_error(tmp_path):
    with pytest.raises(pk.PhytKitError) as e:
        pk.agent_files(tmp_path / "nope")
    assert e.value.code == "file_not_found"


def test_skills_path_and_install(tmp_path):
    code, res = pk_cli("skills", "path")
    assert code == 0 and Path(res["skill"]) == SKILL.resolve() or Path(res["skill"]).samefile(SKILL)
    dest = tmp_path / "skills"
    code, res = pk_cli("skills", "install", "--to", str(dest))
    assert code == 0 and [s["status"] for s in res["skills"]] == ["installed"]
    for f in SKILL.parent.rglob("*.md"):
        assert (dest / "phytkit" / f.relative_to(SKILL.parent)).read_bytes() == f.read_bytes()
    assert (dest / "phytkit" / ".phytkit-managed").is_file()
    # A copy pk installed is replaced; a folder it didn't install is refused without --force.
    assert pk_cli("skills", "install", "--to", str(dest))[1]["skills"][0]["status"] == "replaced"
    (dest / "phytkit" / ".phytkit-managed").unlink()
    code, res = pk_cli("skills", "install", "--to", str(dest))
    assert code == 1 and res["error"]["code"] == "already_exists"
    code, res = pk_cli("skills", "install", "--to", str(dest), "--force")
    assert code == 0 and res["skills"][0]["status"] == "replaced"


def test_default_install_goes_to_claude_skills(tmp_path):
    code, res = pk_cli("skills", "install", cwd=tmp_path)
    assert code == 0 and (tmp_path / ".claude" / "skills" / "phytkit" / "SKILL.md").is_file()
