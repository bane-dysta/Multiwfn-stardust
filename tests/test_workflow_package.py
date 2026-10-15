"""Run the extracted native sidecar, module library and engine on every OS.

Only XYZ I/O is executed; density and Mayer recipes are rendered in dry-run mode.
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from test_workbench_cli import check as check_workbench
from test_workflow_shell import check as check_shell


def check(root):
    root = root.resolve()
    suffix = ".exe" if os.name == "nt" else ""
    runner = root / "resources/tools" / ("multiwfn-workflow" + suffix)
    engine = root / ("Multiwfn_MatterVizGUI" + suffix)
    library = root / "resources/workflows"
    check_workbench(runner)
    check_shell(runner)
    for required in (runner, engine, library / "LICENSE", library / "NOTICE.md"):
        assert required.is_file(), f"Missing package file: {required}"
    with tempfile.TemporaryDirectory(prefix="workflow package ") as directory:
        work = Path(directory)
        startup = work / "single atom.xyz"
        startup.write_text("1\nPackage fixture\nHe 0 0 0\n", encoding="utf-8")
        (work / "banewfn.rc").write_text(
            f'Multiwfn_exec="{engine.as_posix()}"\n'
            f'confpath="{(library / "conf").as_posix()}"\n'
            'cores=1\ncitations_output=off\n', encoding="utf-8",
        )

        def run(script, *options):
            result = subprocess.run(
                [str(runner), str(script), "--wfn", str(startup), *options],
                cwd=work, stdin=subprocess.DEVNULL, capture_output=True,
                text=True, errors="replace", timeout=30,
            )
            assert result.returncode == 0, result.stdout + result.stderr

        for name in ("density.bw", "mayer.bwc"):
            run(library / "scripts" / name, "--dryrun")
        generated = "\n".join(p.read_text() for p in work.glob("*.txt"))
        assert "9\n1\ny\n0\nq" in generated, generated
        assert "5\n1\n" in generated, generated
        script = work / "export.bw"
        script.write_text("%raw\nxyz\nexported.xyz\nq\nend\n", encoding="utf-8")
        run(script)
        output = (work / "exported.xyz").read_text()
        assert output.splitlines()[0].strip() == "1", output
        assert "He" in output, output
    print(f"Workflow package passed: {root}")


if __name__ == "__main__":
    check(Path(sys.argv[1]))
