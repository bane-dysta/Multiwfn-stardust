# BaneWfn workflow integration — 2026-10-06

## Independent BaneWfn workbench — 2026-10-15 (in verification)

Baseline remains `ea0befc3aad728bdd61ede9b45a68725df490f5d`; prior work is
preserved, including the tested `e73c37f3ac576529424ec1e4de4645d35f7d2031`
snapshot. The new entry uses the existing Rust/Tauri + Svelte framework and
starts a batch Multiwfn only when a workflow runs. It does not require a molecule
manifest or a persistent session. The scientific sources are unchanged.

Canonical BaneWfn 1.6.0rc1 implements conf comment annotations, JSON inspection
and scoped overrides; these are synchronized into the embedded runner with the
existing shell/path and session adapters retained. The UI uses parser-generated
fields, separate repeated-step values, a native-input preview and ordinary
Windows path display. Linux/Windows launchers and macOS packaging add an
independent BaneWfn entry.

Local validation: frontend type check/build passed; standalone and existing
panel browser tests passed. Headless Rust: 145 passed, 1 live test intentionally
ignored, clippy clean. The first sandbox run could not open sockets and was
terminated; the same tests passed with local socket access. Core CLI: 4 CTest
targets passed, including inspection/dry-run parity and inline conf tests.
Cross-platform Actions and native Windows chemistry validation are pending.

Local entry: `build/banewfn`. UI screenshots: `build/workbench-ui/`.

## Windows runner path compatibility — 2026-10-13 (accepted)

Release: `workflows-winpath1`. Code commit
`e73c37f3ac576529424ec1e4de4645d35f7d2031`, independent branch
`ci/banewfn-windows-path-20261013`. Actions run:
https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38379097903 .
All four test/package jobs passed; tag-only publication/signing were skipped.

The user's real Windows job records exposed two boundary failures: engine-first
runs passed a canonical `\\?\` executable path to cmd.exe, and desktop-owned
runs passed a verbatim conf directory to the C++ loader, which appends
`/grid.conf`. The former failed to start and the latter could not open the
configuration. The inline `.bwc` and raw-block session checks previously missed
the external-conf case; the standalone batch check manually supplied normalized
paths and missed host-generated configuration entirely.

The host now converts only `Multiwfn_exec` and `confpath` in its generated rc file
to ordinary Windows drive/UNC paths. Canonical paths remain in host file I/O,
containment checks and startup/session identity; moved or removed source files
still do not trigger a reload. No C++ parser or scientific code is changed.

Validation completed locally:

- On the user's real Windows host, a cross-compiled headless harness ran the
  production service against the existing downloaded native engine and runner.
  Both batch and persistent external-conf XYZ export passed; inline modules,
  retained state after source removal, cancellation, UTF-8 reload and GUI
  transport also passed (PID 12572, 4.92 s). Independent check directory:
  `C:\Users\Bane\AppData\Local\Temp\multiwfn-path-fix-20261013-2`.
- Windows drive/UNC/Chinese path unit test: 1 passed. Linux headless suite:
  144 passed; extended live test: passed (PID 2588585, 2.44 s).
- Complete local GUI build and headless clippy passed. Local checks use only
  native menu state and XYZ/Cube I/O; benzene Mayer/density computation was not
  rerun locally and the user's existing inputs/results were not modified.
- Evidence: `build/windows-path-live2.log`, `build/windows-path-linux-unit.log`,
  `build/windows-path-linux-live.log`, `build/windows-path-local-build.log`.

Final package validation:

- Windows CI: 116 host tests and 120 headless tests passed, including the new
  path unit test. Extracted-package batch/persistent external-conf regression
  passed (PID 7612, 8.78 s). Linux and macOS packaging and tests also passed.
- The downloaded Windows ZIP matches GitHub's published SHA-256
  `2669432d18e2053b981c821584a2504e9a8af8dda702a786d978ededf8c8e02a`.
  It was unpacked into the new `C:\Users\Bane\Downloads\Multiwfn-Windows-winpath1`
  folder; the previous package was not overwritten.
- Real Windows WebView on this host: external `.bw` and inline `.bwc` XYZ
  exports completed, the shipped `density.bw` prepared successfully, all using
  engine PID 35344. Return closed the GUI with exit code 0. No scientific
  calculation was requested. Evidence: `build/windows-path-real-gui.log` and
  `C:\Users\Bane\AppData\Local\Temp\multiwfn-winpath-gui-1791886602927`.
- Final local Linux host was rebuilt/staged after the fix. All 68 original
  scientific files remain unchanged. Delivered sources differ from the CI
  commit only in this record and workflow documentation.

Ready to test: `C:\Users\Bane\Downloads\Multiwfn-Windows-winpath1\Multiwfn.exe`.
[Windows download](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38379097903/artifacts/11803189170).
Delivery: `artifacts/2026-10-13/deliveries/multiwfn-winpath1/`, cumulative patch
and incremental patch from `linux-entry1`, checked on freshly extracted sources.
WIP-026/027/028: `artifacts/2026-10-13/wip/multiwfn-windows-path/`.

## Linux application entry — 2026-10-13 (accepted locally)

Release: `workflows-linux-entry1`. Linux workflow builds now include the compiled
`Multiwfn` GUI entry, a project icon, `Multiwfn-Workflows.desktop` in the build
directory, and `install-desktop-entry.sh` for the current user's application
menu. The entry finds its own directory through `/proc/self/exe`, preserves file
arguments and exit status, and delegates to the already validated runtime
wrapper. The persistent engine and original scientific code are unchanged.
The Linux CI package now includes this entry, icon and registration script;
its package job exercises the relocated executable through GIO. Registration
uses absolute paths and must be repeated if the package is moved.

Local ready-to-use entries:

- Executable: `/home/bane/smokes/Multiwfn-stardust/build/Multiwfn`.
- Linux menu: `/home/bane/.local/share/applications/Multiwfn-Workflows.desktop`.
- This WSL host also has a Windows Start-menu shortcut **Multiwfn (Linux)**,
  with a copy at `build/Multiwfn (Linux).lnk`. It runs the same Linux executable
  using `C:\Program Files\WSL\wslg.exe`, distribution `Ubuntu-24.04`, user `bane`.
  This machine-specific shortcut is outside the portable source/package.

Validation:

- `cmake -S . -B build` and `cmake --build build -j4`: passed; reused the
  existing Release build and staged native/frontend resources.
- `python3 -m unittest discover -s tests -p test_linux_desktop_entry.py -v`:
  3 passed, including real GIO launch with no input and with a startup file,
  relocation, symlinks, Chinese/spaces/special characters and failure handling.
- `python3 -m unittest discover -s tests -p test_matterviz_build_names.py -v`:
  25 passed. `python3 tests/test_workflow_package.py build`: passed (XYZ I/O
  and recipe rendering only, no electronic-structure calculation).
- Actual installed Linux menu entry opened a WSLg WebView with a Chinese/space
  startup filename, emitted `ready`, and reaped engine PID 2560134 after Return.
  Evidence: `build/linux-entry-native-window/`. Windows shortcut target and
  arguments were checked after saving; direct Windows shortcut clicking was
  not automated. Windows/macOS engines were not rebuilt in this round.
- C launcher compiled with `-std=c11 -Wall -Wextra -Werror`; shell syntax,
  workflow YAML parsing, `git diff --check`, and staged CMake workflows install
  passed. All 68 original scientific source files remain identical to baseline.
- This entry-only change has not been run on GitHub Actions yet; the preceding
  three-platform session implementation's CI results remain recorded below.

Delivery: `artifacts/2026-10-13/deliveries/multiwfn-linux-entry1/` (cumulative and
session2-ci1 incremental patches, source archive, WIP bundle and checksums).
WIP-024/025: `artifacts/2026-10-13/wip/multiwfn-linux-entry/`.

## Portable persistent sessions — 2026-10-13 (accepted)

Release: `workflows-session2-ci1`. Final commit
`a32b50776fa7465097fde520b18aaaab422256b2`; all four test jobs passed in
https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38369412491 .
Tag-only signing/publication were skipped. No upstream PR or merge was made.

Windows and macOS now use the desktop-owned session implementation alongside
Linux. The generated entry still has exactly two adapter substitutions; original
scientific sources are not edited. Protocol MWS2 carries complete command blocks
and acknowledges block IDs. Fortran supplies its own scratch input stream and
measures its boundary using runtime positions, removing the /proc dependency
and wire-byte/newline assumptions. Windows private pipe handles are explicitly
duplicated into the intended child and delivered through bootstrap stdin; they
are not made globally inheritable. Unix endpoints use close-on-exec discipline.

Windows packages add the desktop entry `Multiwfn.exe`. Workflow-enabled macOS
apps use desktop-first launch, preserving the CLI engine and workflow-disabled
app behavior. macOS app resources and executable paths are resolved separately.
Engine startup logs now live under the user's temporary directory. Owned engine
destruction also reaps a child after startup/service failure.

Windows engine and runner embed a UTF-8 process manifest. The extended second
run (38367054540) exposed a native reload failure for Chinese filenames; this
manifest fixes the code-page mismatch without changing original Fortran sources.
A probe on the user's Windows host measured ACP 936 without the manifest and
65001 with it. The final native Windows test passes Chinese reload and subsequent
workflow staging. The source-build Windows launcher is installed with workflows,
and the CMake macOS app target supplies workflow resources when enabled.

Validation:

- Linux host/headless suites: 140/144 passed; Windows: 115/119; macOS: 133/137.
  The two new pipe tests cover fragmented 256 KiB transfers, EOF and timeout.
- Extracted-package persistent tests passed: Windows PID 8896 (7.87 s), Linux
  PID 30220 (2.22 s), macOS tar PID 31949 (5.16 s) and app PID 32038 (5.32 s).
  Linux build-tree PID 32671 also passed (2.29 s). These explicitly enable the
  live test, which is ignored in the ordinary unit-test invocation.
- Live tests cover the same PID across workflows, retained menu/grid state after
  removing source files, raw exit rejection and continued use, cancellation,
  explicit/same-path reload, GUI refresh, automatic results, CRLF recipes,
  a 96 KiB command block, Chinese/space paths, and clean exit code 0.
- The Fortran scratch-input probe passed on all three platforms. Existing batch
  package tests and GUI/IPC regressions also passed, including macOS tar/app.
- Local full build: `bash tools/build_workflows_gui.sh`; final incremental build:
  `cmake --build build -j4`. Local native menu/I/O test PID 2498139 passed (2.29 s).
  A real WSLg WebView opened and returned exit code 0 through the authenticated
  Return endpoint. Local checks included 142 existing native tests, two new pipe
  tests, 9 C++ runner tests, build/entry/app contracts, clippy, Windows GNU Rust
  cross-check, Windows C adapter/launcher compilation, and manifest compilation.
- All 68 original scientific sources remain byte-identical to the baseline.
  Local live tests perform menu/data I/O, not electronic-structure calculations.

Full Windows/macOS desktop interaction and their native startup file dialogs
have not been manually exercised here; CI validates their native session engines,
packaged resources and launch arguments. Original Multiwfn filename storage is
200 bytes (`define.f90`); that native limit is unchanged. Cancellation waits for
a block to return to the main menu; arbitrary pre-existing terminal engines cannot
be attached. These remain the declared boundaries of this adapter.

Downloads (GitHub login required):

- [Windows ZIP](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38369412491/artifacts/11798544717): run `Multiwfn.exe`.
- [macOS DMG](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38369412491/artifacts/11799266494): open `Multiwfn.app`.
- [macOS tar](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38369412491/artifacts/11798967916).
- [Linux tar](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38369412491/artifacts/11798188685).

Complete local executable:
`/home/bane/smokes/Multiwfn-stardust/build/run-workflows-gui [startup-file]`.
Engine: `build/Multiwfn_MatterVizGUI`; desktop/runner: `build/resources/tools/`.
Delivery: `artifacts/2026-10-13/deliveries/multiwfn-session2-ci1/` (cumulative and
session1-ci1 incremental patches, source, WIP bundle, validation and checksums).
Logs: `artifacts/2026-10-13/checks/multiwfn-session2/`.
WIP-020 onward: `artifacts/2026-10-13/wip/multiwfn-session2/`.
Final delivered executable sources match the passing commit; only this acceptance
record is added afterward. No incomplete binary downloads are included.

## Cross-platform Actions validation — 2026-10-13

Release designation: `workflows-session1-ci1`. Final Actions run
https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38359571705
completed **success** at commit `fa026cceef74b5a9f3ea4148b3ac3231c1a8d81d`
on branch `ci/banewfn-session-20261013-final`. All four test jobs passed:
Linux frontend/Rust/native checks and Linux, Windows, macOS package jobs.
Tag-only signing and release publication were skipped as intended.

The `matterviz-gui` workflow now builds with `MULTIWFN_WITH_WORKFLOWS=ON`
on Linux, Windows and macOS. Packages include the C++ runner, conf library,
starter scripts and license; the macOS app/DMG includes the same resources.
Extracted-package checks render both `.bw` and `.bwc` recipes, then perform
single-atom XYZ export through the real bundled engine. Linux additionally
checks persistent session state and GUI transport against both the build tree
and the extracted package. Windows/macOS still use the batch backend; their CI
success does not imply support for the Linux persistent-session backend.

Local checks: 62 build/adapter/package-contract tests passed; extracted-layout
workflow test passed against `build/`; real persistent-session test passed
(final full-build check: PID 2446725, 1.94 seconds). Socket tests ran outside the
sandbox. Full local build used `bash tools/build_workflows_gui.sh`; the final
Windows-only quoting correction was rebuilt with `cmake --build build --target
multiwfn-workflow --parallel 4` and staged into `build/resources/tools/`.
Both the 9 C++ runner tests and the native package XYZ check passed afterward.
The 9 workflow-host tests passed with a symlinked temporary directory and the
real C++ runner enabled. All 68 scientific sources match the original baseline.
Rust formatting, workflow YAML parsing, and syntax of all 34 shell steps passed.
Local new tests only exercise menu state and XYZ/Cube I/O; remote CI additionally
runs the repository's existing native GUI/orbital and functional regressions.

CI fixes:

- Canonicalize the test fixture root: macOS `/var` resolves to `/private/var`,
  matching the paths returned by production native selection.
- Explicitly select C++17 for the runner. GCC 16's newer default makes the
  vendored `u8` literals incompatible with `std::string`; minimum-feature
  selection alone did not constrain the language dialect.
- Wrap generated Windows engine command lines in the outer quotes required by
  `cmd.exe /c`. A Windows UCRT probe returned 1 before and 0 after the correction;
  the final extracted Windows package now successfully runs the real engine.

Earlier diagnostic runs `38356482459` and `38358329798` are retained. Logs,
job/artifact metadata and patch checks are in workspace-relative
`artifacts/2026-10-13/checks/multiwfn-ci/`. WIP-015 onward is in
`artifacts/2026-10-13/wip/multiwfn-ci/`. Cumulative and session1-r2 incremental
patches, source archive, WIP bundle and checksums are in
`artifacts/2026-10-13/deliveries/multiwfn-session-ci1/`.

CI binary downloads (GitHub login required):

- [Windows ZIP](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38359571705/artifacts/11794582506)
- [Linux tar.gz](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38359571705/artifacts/11794144885)
- [macOS tar.gz](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38359571705/artifacts/11794673233)
- [macOS DMG](https://github.com/bane-dysta/Multiwfn-stardust/actions/runs/38359571705/artifacts/11794962455)

These are complete cloud artifacts. Optional local downloads were stopped due to
low throughput; partial downloads under `build/ci-incomplete-downloads/` are not
deliverables. The complete local Linux/WSL GUI remains ready to run at
`/home/bane/smokes/Multiwfn-stardust/build/run-workflows-gui [startup-file]`.
Only this status record differs from the tested commit in the final source
delivery; executable and CI sources are identical to the passing revision.

## Current delivery: persistent Linux session — 2026-10-13

Release designation: `workflows-session1-r2`. Cumulative baseline remains
`ea0befc3aad728bdd61ede9b45a68725df490f5d`; incremental baseline is preview4
tree `879f54c0dec88d8367606a8861aa86e0385ddae7`.

**Runnable desktop entry:**
`/home/bane/smokes/Multiwfn-stardust/build/run-workflows-gui [startup-file]`.
Omit the argument for the file chooser. Engine:
`build/Multiwfn_MatterVizGUI`; desktop and C++ runner:
`build/resources/tools/matterviz-desktop` and `multiwfn-workflow`.
These are a complete native GUI build, not only frontend HTML assets.

The launcher owns one Multiwfn process. BaneWfn sends expanded blocks through a
private inherited Unix socket; the native adapter schedules stdin at main-menu
boundaries. A generated entry copy adds initialization and replaces the main
menu read. All 68 original scientific files remain byte-identical to baseline.
No wavefunction/grid reload occurs between ordinary blocks or workflows.
Explicit GUI selection and `wfn_rebase=` use the native reload menu, including
same-path rebase. Each completed workflow refreshes structure and grid transport;
automatic results roots follow the current file until manually overridden.

Cancellation waits for the active block, skips subsequent blocks, and keeps the
engine alive. Closing the owned GUI waits for cancellation and closes the engine.
Literal main-menu exit is rejected; generated final quits are removed only from
module/builtin expansion. `wait`, debug and unresolved interactive variables are
unsupported. Scripts must return to the main menu; unexpected submenu input has
no interactive recovery in this first release. Automatic blocks suppress GUI
openings. Arbitrary pre-existing terminal processes cannot be attached.

Build and verification:

- `bash tools/build_workflows_gui.sh`: full frontend/Rust/C++/Fortran build using
  the validated `build/` CMake cache, Release, concurrency 4. Svelte check reports
  0 errors/warnings. Existing CSS/chunk-size build warnings remain.
- Rust headless production-module suite: 142 passed. Live ignored-by-default
  test explicitly enabled: 1 passed in about 2 seconds, engine PID 2358602.
  It verifies retained menu state and structure after removing the source,
  inline `.bwc` final-quit handling, rejected raw exit and continued use,
  cancellation after a complete block, explicit same-path reload, native Cube
  loading followed by source removal and all eight retained grid values, GUI
  refresh, automatic results, and clean process shutdown. These are menu/data
  I/O checks; no electronic-structure calculation or scientific analysis ran.
- Rust clippy `--all-targets -- -D warnings`: passed. Existing C++ runner tests
  with a mock engine: 9 passed. Build integration tests: 25 passed. Generated
  entry anchor/byte-preservation tests: 2 passed.
- Focused frontend workflow/i18n tests: 8 passed. Existing Playwright acceptance
  passed (30/70 columns, three tabs, parameter persistence, startup/results
  defaults, no engine picker, preview/run/cancel, Chinese and narrow screens).
- Actual WSLg native desktop window rendered the loaded structure and emitted
  the WebView `ready` signal. `native-window.png` and desktop startup logs record
  that separate native check; browser API simulation is not counted as native
  execution evidence.
- A second Playwright check uses the actual final desktop HTTP service, without
  mocked routes: it verifies the harmless fixture source, clicks Run, observes
  Completed and unchanged PID 2364952, and saves the refreshed Results panel.
  Evidence: `browser-real-final.log` and
  `browser-real-final/real-session-workflow.png`. Final layout regression also
  passed against the final assets (`browser-final.log`).
- r2 native picker validation: launch without arguments, open the GTK chooser,
  send the known XYZ path through X11 keyboard events, accept selection, and
  observe the desktop ready signal (PID 2382147). Evidence:
  `native-picker.png`, `picker-launch-final.log`,
  `picker-launch-final-ready.status`. Folder selection has not been automated.
- Independent `/home/bane/smokes/banewfn` remains unchanged. No upstream push,
  issue comment or PR was created. Windows/macOS session execution and real
  scientific result equivalence remain unverified.

Logs and evidence: workspace-relative
`artifacts/2026-10-13/checks/multiwfn-session/`, especially
`live-final/session-validation.json`, `native-tests-final.log`,
`clippy-final.log`, and the final full-build log. Earlier failed checks are
retained: they caught a disk-existence check and a grid-bootstrap refresh bug,
both covered by the final live test.

The host lacks system WebKit development/runtime packages. Ubuntu debs were
verified and extracted into `build/gui-sysroot`; no system packages were installed.
The launcher can expose local WebKit helpers with the installed `bwrap` in a
private mount namespace. This dependency bundle is for Ubuntu 24.04 x86-64/WSL,
not a universal binary distribution. Native library warnings did not prevent
the verified structure rendering. Linux still needs a graphical session.

WIP-011 onward: `artifacts/2026-10-13/wip/multiwfn-session/`.
Cumulative/incremental patches, source, local runtime bundle and checksums:
`artifacts/2026-10-13/deliveries/multiwfn-session-r2/`. The earlier session1
delivery is retained unchanged; r2 additionally provides the Linux GTK chooser
helper for systems such as this WSL host without a desktop portal. It runs in
a separate process and leaves the existing portal backend as the fallback.
Older releases below record
their historical batch behavior; this section describes the current launcher.

Baseline: `ea0befc3aad728bdd61ede9b45a68725df490f5d` (clean working tree).
The source BaneWfn repository is read-only for this task.

## Plan and boundaries

1. Bundle an optional C++ BaneWfn batch runner and its existing module library.
2. Add a workflow panel to the current MatterViz/Svelte workbench.
3. Have the Rust desktop host authorize native selections and manage independent
   child processes, fresh result directories, logs, exit status and cancellation.
4. Validate DSL and process behavior with a mock Multiwfn, check the frontend and
   native host modules, and deliver an applicable Git patch plus source archive.

Only frontend, adapter/build integration, tests and documentation are in scope.
Root Fortran sources, `ext/` and `libreta_hybrid/` must stay identical to baseline.
Desktop workflows use batch `end`; terminal `wait` is rejected explicitly.

## Maintainer preferences

- Issue #58 records BaneWfn's existing stdin orchestration design:
  https://github.com/Stardust0831/Multiwfn/issues/58
- PR #27 establishes external orchestration, optional dependencies and scientific
  ownership in Multiwfn: https://github.com/Stardust0831/Multiwfn/pull/27
- PR #26 demonstrates the current Rust native host / MatterViz process boundary:
  https://github.com/Stardust0831/Multiwfn/pull/26
- `CONTRIBUTING.md` favors focused adapter changes. A review-ready issue/PR draft
  will be included in the integration documentation; no upstream message is sent.

## Progress

Implemented the optional C++ runner, two starter workflows, the native script
library/preview/picker/run/cancel API, process-tree ownership, fresh job staging,
bounded logs, terminal status and the bilingual workbench panel. Build/usage and
review scope are documented in `docs/workflows.md`; the CI gate covers the runner.

WIP-001 through WIP-004 are saved under
`artifacts/2026-10-06/wip/multiwfn-workflows/`, relative to the workspace root.
All include newly added files and are cumulative against the baseline above.

## Validation

- `cmake -S frontend/workflows -B build/workflow-runner -DCMAKE_BUILD_TYPE=Release`
  and `cmake --build build/workflow-runner -j4`: passed.
- Top-level CMake configuration with `MULTIWFN_WITH_WORKFLOWS=ON`, followed by
  building only `multiwfn-workflow`: passed. The optional install component was
  staged successfully with the runner, 26 module configs, scripts and licenses.
- `python3 -m unittest discover -s tests -p test_workflow_runner.py -v`:
  9 passed, no skips. This uses a mock Multiwfn, including stdin, quoted paths,
  module/inline config, variable overrides, preflight, exit propagation and grep.
- Existing `test_matterviz_build_names.py`: 25 passed.
- Frontend `npm run check`: zero errors/warnings; `npm run build`: passed.
- Node 24.11.1 full frontend tests with `--test-concurrency=4`:
  298 passed, 1 pre-existing skip (vector PDF requires a browser DOM), no failures.
- Existing headless Rust production-module/C-ABI harness, including workflow
  tests and the real C++ sidecar with a mock engine: 136 passed, no ignored tests.
  `cargo clippy --locked -- -D warnings` and Rust formatting checks passed.
- Windows C++ runner cross-build with MinGW/static runtimes: passed.
  Rust host modules cross-check for `x86_64-pc-windows-gnu`: passed.
- A byte comparison against Git baseline found all 68 official core files
  unchanged, including root Fortran, `ext/` and `libreta_hybrid/`.

The initial sandboxed frontend run rejected nested Node processes with `EPERM`,
and one sandboxed native test stalled. Re-running the existing tests outside the
sandbox passed; no baseline test was altered to suppress either problem.

- Playwright acceptance passed for script preview, literal source rendering,
  picker API, parameters, preflight, run, cancellation, Escape, reopening,
  400-pixel layout and Chinese controls. Screenshots are in the workspace's
  `artifacts/2026-10-06/checks/multiwfn-workflows/browser/` directory. Browser
  selection APIs were simulated; the production Rust process manager and real
  C++ sidecar were exercised separately in the native integration tests.

The cumulative Git patch passed `git apply --check` and was applied to a fresh
archive of the baseline. Delivery includes a cumulative patch, an incremental
patch against the WIP-004 source tree, both source archives, a WIP patch bundle,
review notes and SHA-256 checksums. The source archives contain tracked source
and all added files; build outputs, tooling, caches and `.git` are excluded.

No real scientific calculation was started. The full Tauri WebView build/native dialogs,
Windows/macOS runtime behavior and real scientific output remain unverified on
this host. macOS release bundling of the optional resources is a separate step.

## Delivery

Release designation: `workflows-preview1`; this is a reviewable optional feature,
not a change to Multiwfn's scientific source version. Baseline and limitations
are recorded above. Files are under the workspace's
`artifacts/2026-10-06/deliveries/multiwfn-workflows/` directory. The independent
source BaneWfn repository remains unchanged. No remote branch, issue or PR was
published.

## Panel layout revision — 2026-10-07

Release designation: `workflows-preview2`. Incremental baseline is the delivered
`workflows-preview1` source archive (tree
`6573432e0e3647a8d732c3d434fef5f5433c6718`); the cumulative baseline remains
`ea0befc3aad728bdd61ede9b45a68725df490f5d`.

The panel now has two bounded columns: script library and source preview on the
left, run settings/actions and output on the right. Script buttons show selection;
the preview has line numbers. Log and result-file views share one output area.
Long logs scroll there without expanding the window. Picked paths occupy their
fields and expose full paths on hover. Idle cancellation controls and permanent
batch/variable/runtime helper text are removed. Typography, icons, spacing and
colors use the existing workbench palette. Narrow windows stack the columns.

This revision changes only `WorkflowPanel.svelte`, UI translations, its browser
acceptance test, usage documentation and this status record. The workflow client,
Rust host, C++ runner and scientific implementation retain preview1 behavior.
WIP-005 and WIP-006 are under the workspace's
`artifacts/2026-10-07/wip/multiwfn-workflows-layout/` directory.

Validation on the revised panel:

- Frontend `npm run check`: zero errors/warnings. `npm run build`: passed;
  existing dependency/plugin timing and large-chunk build warnings remain.
- Node 24.11.1 `node --experimental-strip-types --test-concurrency=4 --test
  tests/workflows.test.ts tests/i18n.test.ts`: 8 passed, no skips/failures, outside
  the sandbox so the Node child processes actually execute the test cases.
- Playwright `node tests/browser/workflows.mjs`: passed. Checks cover separate
  aligned columns, unchanged window dimensions after preflight, bounded long
  logs, 1024x768 laptop layout, script selection/preview, literal source text,
  picker API, variables, preflight/run/cancel, log/file switching, Escape,
  reopening, Chinese controls and a 400px layout without horizontal overflow.
  Browser APIs are simulated; no scientific computation was started.
- Repeated byte comparison confirms all 68 official scientific core files
  unchanged from the cumulative baseline. The source BaneWfn repository remains
  clean. Native integration checks from preview1 were not rerun for this UI-only
  revision; their original scope and limitations above still apply.

New screenshots, logs, cumulative and incremental patches, source archive and
WIP bundle are delivered under the workspace's
`artifacts/2026-10-07/deliveries/multiwfn-workflows-layout/` directory, with separate
patch application and archive validation notes. Prior deliveries are preserved.

## Workflow tabs revision — 2026-10-07

Release designation: `workflows-preview3`. Incremental baseline is the delivered
preview2 source tree `1ab298bbcb6ba12e8c30ecb31a475d56464a9c17`. The original
cumulative baseline remains `ea0befc3aad728bdd61ede9b45a68725df490f5d`.

The left column now occupies 30% of the desktop panel and contains only the
script library. The right 70% uses the workbench's existing Tabs components for
three pages: Parameters, Script preview and Results. Parameters means graphical
input of workflow settings: wavefunction, results directory, Multiwfn engine,
cores and variable overrides. Script selection and filled values survive tab
changes. The shared footer keeps preflight/run/cancel available on every page;
successful preflight/run opens Results. Active jobs keep polling across pages.
The preview retains line numbers, while Results retains log/file views and
bounded log scrolling. Narrow windows put the library above the right-side pages.

Only the panel, translated labels, browser acceptance test, usage documentation
and this record changed since preview2. WIP-007 and WIP-008 are stored under
`artifacts/2026-10-07/wip/multiwfn-workflows-tabs/`, relative to the workspace.

Validation:

- `npm run check`: 0 errors/warnings.
- `npm run build -- --outDir ../../build/matterviz-viewer`: passed; existing
  plugin timing and large-chunk build warnings remain. The exact compiled entry
  is `/home/bane/smokes/Multiwfn-stardust/build/matterviz-viewer/index.html`.
  These assets were also copied into `frontend/matterviz-viewer/dist` for the
  existing native packaging path, and copied files were compared byte-for-byte.
- Focused Node tests (`tests/workflows.test.ts`, `tests/i18n.test.ts`), Node
  24.11.1 with concurrency 4 outside the sandbox: 8 passed, no failures/skips.
- Browser acceptance ran against the compiled assets served by Vite preview.
  Passed: 30/70 split, exactly three tabs, keyboard arrow/Home/End navigation,
  preview in the right column, parameter persistence, shared actions, automatic
  Results after preflight/run, long-log containment, picker API, safe text
  rendering, cancellation from preview, Escape/reopen, Chinese controls and
  1024x768/400px viewports. Native browser APIs were simulated.
- All 68 scientific core files still match the original baseline byte-for-byte.
  The independent BaneWfn source repository remains clean.

Native host and runner checks were not repeated for this UI revision. No real
calculation or full native WebView build ran; the native/platform limitations
recorded for preview1 continue to apply. Preview3 deliveries and patch-application
notes are under `artifacts/2026-10-07/deliveries/multiwfn-workflows-tabs/`.

## Startup file and current engine — 2026-10-07

Release designation: `workflows-preview4`. Incremental baseline is preview3
source tree `16c6169c890ce18eb6c1341b91cb26ae03f4915e`; the cumulative baseline
remains `ea0befc3aad728bdd61ede9b45a68725df490f5d`.

The panel now calls the input **Startup file** and removes the Multiwfn picker.
The native host resolves the executable of the launcher's Multiwfn PID on Linux,
Windows and macOS and keeps it immutable. It no longer uses the separate engine
environment override or searches for another installed build. The selection API
also rejects engine replacement. An unresolved current executable blocks running
with a readable error instead of substituting another engine.

Two presentation-only lines in `noGUI/GUI_matterviz.f90` add the loaded filename
to structure/plot session metadata. The host uses this metadata to prefill the
startup file and `<input directory>/<input stem>_workflows` results root. Selecting
a different input updates the automatic root; explicitly choosing a results
directory preserves that choice on subsequent input changes. Output directories
are created at preflight/run time. Marked results roots are excluded from the
script catalog and input copying, so results may be beside scripts without
recursively staging old or current output trees. Existing outputs are retained.

The browser-facing status retains its existing `wavefunction` and `multiwfn`
fields for compatibility; UI terminology changes and the executable is read-only.
Changes cover the native workflow manager/service/picker, panel/client labels,
browser acceptance, starter-script wording, usage docs, the metadata adapter and
this record. Official root Fortran, `ext/`, and `libreta_hybrid/` remain unchanged.
WIP-009 and WIP-010 are in `artifacts/2026-10-07/wip/multiwfn-workflows-startup/`.

Validation:

- Production-module/C-ABI native harness with the real C++ sidecar and a mock
  engine: 142 passed, 0 failed/ignored. New cases cover file/memory session
  prefill, automatic results, manual override, results exclusion after repeated
  inputs/runs, actual process-image lookup and refusal of engine substitution.
- Native `cargo clippy --locked --offline --all-targets -- -D warnings` and
  Rust formatting checks passed. Windows GNU production/test-target cross-check
  passed. All use isolated Rust 1.88.0 tooling and existing build caches, `-j4`.
- Existing build/GUI integration checks: 25 passed, including source/build-name
  guards. The full Fortran GUI adapter was not compiled in this revision.
- Frontend check: 0 errors/warnings; production build passed. Compiled entry:
  `/home/bane/smokes/Multiwfn-stardust/build/matterviz-viewer/index.html`.
  Assets were also staged to `frontend/matterviz-viewer/dist` and compared.
- Workflow client and bilingual text tests: 8 passed, no failures/skips.
- Browser acceptance against compiled assets passed: startup terminology,
  initial loaded-input/result prefill, automatic results after input selection,
  manual-directory preservation, absent engine picker, prior 30/70 layout,
  tabs/keyboard/parameter persistence, preflight/run/cancel, bounded logs,
  1024x768/400px layouts, safe text rendering and Chinese controls.
- All 68 official scientific core files match the original baseline byte-for-byte.
  The independent BaneWfn source repository remains clean.

Tests execute no scientific calculation. Browser APIs are simulated; native
ownership, staging and the real runner are covered separately by the harness.
The full Tauri application/GUI adapter rebuild, real native file dialogs, Windows
runtime and macOS process lookup remain unverified. The host lacks pkg-config
for a full native GUI build; no system dependency installation was attempted.
To use all new behavior in a desktop package, rebuild the native host and GUI
adapter as well as updating frontend assets. New cumulative/incremental patches,
source, frontend assets, screenshots and validation notes are delivered in
`artifacts/2026-10-07/deliveries/multiwfn-workflows-startup/`.
