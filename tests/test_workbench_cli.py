"""Exercise inspection and execution through the public CLI, without Multiwfn."""
import json
import pathlib
import subprocess
import sys
import tempfile

def check(executable):
    exe = pathlib.Path(executable).resolve()
    with tempfile.TemporaryDirectory(prefix="banewfn-gui-") as tmp:
        root = pathlib.Path(tmp)
        conf = '''#@gui grid
    # label = 网格精度
    # type = select
    # options = 1:低 | 2:中 | 3:高
    #@end
    [main]
    5
    [electron]
    ${grid:-3}
[spin] # section comment
    #@gui grid
    # label = 自旋网格
    #@end
    ${grid:-2}
    [quit]
    q
    '''
        (root / "grid.conf").write_text(conf, encoding="utf-8")
        (root / "banewfn.rc").write_text(f'Multiwfn_exec=unused\nconfpath="{root.as_posix()}"\n', encoding="utf-8")
        script = root / "same.bw"
        script.write_text('x=3\n[grid]\n%process\nelectron grid ${x}\nelectron\nspin\n%command\necho bad > side-effect\nend\n', encoding="utf-8")
        (root / "tiny.xyz").write_text("1\nfixture\nH 0 0 0\n")
        def inspect(overrides=None, target=script):
            args = [str(exe), "--inspect-json", str(target), "--conf-dir", str(root), "--wfn", str(root / "tiny.xyz")]
            if overrides is not None:
                (root / "parameters.json").write_text(json.dumps(overrides, ensure_ascii=False), encoding="utf-8")
                args += ["--parameters-json", str(root / "parameters.json")]
            result = subprocess.run(args, cwd=root, capture_output=True, text=True, encoding="utf-8", timeout=10)
            return result.returncode, json.loads(result.stdout)
        code, base = inspect()
        assert code == 0 and base["runnable"], base
        fields = {f["id"]: f for f in base["fields"]}
        assert [fields[f"0/{i}/grid"]["value"] for i in range(3)] == ["3", "3", "2"]
        assert fields["0/2/grid"]["label"] == "自旋网格"
        assert fields["0/2/grid"]["type"] == "select"
        assert not (root / "side-effect").exists()
        code, modified = inspect({"0/0/grid": "1", "var/x": "2"})
        assert code == 0
        assert modified["tasks"][0]["commands"] == "5\n1\n3\n2\nq\n"
        assert inspect({"0/0/grid": "9"})[0] != 0
        assert inspect({"0/99/grid": "1"})[0] != 0
        assert inspect({"var/unknown": "1"})[0] != 0
        inline = root / "same.bwc"
        inline.write_text(script.read_text() + "\n#>>> BANEWFN_INLINE_CONF_BEGIN grid\n" + "".join("# " + line + "\n" for line in conf.splitlines()) + "#<<< BANEWFN_INLINE_CONF_END grid\n", encoding="utf-8")
        (root / "grid.conf").write_text("[main]\n999\n", encoding="utf-8")
        assert inspect(target=inline)[1] == base
        # A real dry run must use precisely the command generator inspected above.
        inspect({"0/0/grid": "1"}, inline)
        run = subprocess.run([str(exe), str(inline), "--wfn", str(root / "tiny.xyz"), "--dryrun", "--nogui", "--parameters-json", "parameters.json"], cwd=root, capture_output=True, text=True, encoding="utf-8", timeout=10)
        assert run.returncode == 0, run.stdout + run.stderr
        native = list(root.glob("grid_*.txt"))
        assert len(native) == 1 and native[0].read_text() == "5\n1\n3\n2\nq\n"
        assert not (root / "side-effect").exists()
        script.write_text("x=?\n[grid]\n%process\nelectron\nend\n", encoding="utf-8")
        (root / "grid.conf").write_text(conf, encoding="utf-8")
        assert inspect()[1]["runnable"] is False
        assert inspect({"var/x": "2"})[1]["runnable"] is True
        (root / "grid.conf").write_text(conf.replace("# type = select", "# type = mystery"), encoding="utf-8")
        assert inspect()[1]["runnable"] is False
    print("GUI CLI: scoped fields, metadata, overrides, inline conf, side effects and dry-run parity passed")


if __name__ == "__main__":
    check(sys.argv[1])
