"""Generate the optional menu adapter; never edit the upstream entry file."""
import argparse
import hashlib
from pathlib import Path


def generate(source: bytes) -> bytes:
    initialization = b"!Special treatment for Intel compiler"
    menu_read = b"\tread(*,*) c200tmp\n"
    if source.count(initialization) != 1 or source.count(menu_read) != 1:
        raise ValueError("Upstream entry changed: session adapter anchors must match exactly once")
    result = source.replace(initialization, b"call matterviz_session_initialize()\n" + initialization)
    result = result.replace(menu_read, b"\tcall matterviz_session_read_main(c200tmp)\n")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    source = args.source.read_bytes()
    result = generate(source)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(result)
    args.output.with_suffix(".sha256").write_text(hashlib.sha256(source).hexdigest() + "\n")
