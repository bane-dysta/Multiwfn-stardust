"""Guard the generated entry against accidental edits during upstream updates."""
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("entry", ROOT / "tools/generate_session_entry.py")
ENTRY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ENTRY)


class SessionEntryTests(unittest.TestCase):
    def test_only_two_menu_adapter_changes(self):
        original = (ROOT / "Multiwfn.f90").read_bytes()
        generated = ENTRY.generate(original)
        restored = generated.replace(b"call matterviz_session_initialize()\n", b"").replace(
            b"\tcall matterviz_session_read_main(c200tmp)\n", b"\tread(*,*) c200tmp\n")
        self.assertEqual(restored, original)

    def test_missing_or_duplicate_anchor_fails_closed(self):
        original = (ROOT / "Multiwfn.f90").read_bytes()
        for anchor in (b"!Special treatment for Intel compiler", b"\tread(*,*) c200tmp\n"):
            for changed in (original.replace(anchor, b""), original + anchor):
                with self.subTest(anchor=anchor, length=len(changed)), self.assertRaises(ValueError):
                    ENTRY.generate(changed)
