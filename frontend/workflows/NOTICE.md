# BaneWfn workflow runner

The sources in `banewfn/src/` and module configurations in `banewfn/conf/` are
derived from BaneWfn by Bane Dysta, version 1.5.6rc5, commit
`9e645fa94581d0c5aa640f3d3b4f2dedc8aed14c`.
Source: https://github.com/bane-dysta/banewfn

GUI annotations, JSON inspection and scoped parameter overrides are synchronized
from the 1.6.0rc1 worktree based on `05dcdbf970e8510ffd410f26568e1ec63f8352ce`.
Their canonical implementation is maintained in the BaneWfn repository.

BaneWfn material retains its Apache License 2.0 in `banewfn/LICENSE`. The
surrounding Multiwfn repository retains its own `LICENSE.txt` and attribution
requirements. Include both licenses when distributing the optional runner.

Local changes cover shell argument and configuration path quoting,
rejecting interactive builtins, the desktop entry point and the optional portable
session execution adapter. That adapter sends expanded command blocks to the
desktop-owned Multiwfn, preserves per-block logs and postprocessing, strips only
generated final termination, and implements explicit reload and cancellation
checkpoints. Trailing whitespace
and final blank lines are normalized in vendored source/config files. The scientific engine remains an
external Multiwfn executable. The desktop host does not parse scientific output
or implement numerical analysis.

When refreshing the vendored snapshot, compare against the pinned commit,
retain these adapter changes, and run the workflow integration tests.
