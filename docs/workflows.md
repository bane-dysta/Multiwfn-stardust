# BaneWfn workflows in the MatterViz workbench

The **Workflows** button opens a script library for `.bw` and `.bwc` workflows.
The left column occupies about 30% of the panel and contains the script list.
The right column has three tabs: **Parameters**, **Script preview** and
**Results**. Run controls stay below all three pages. Preflight and run switch to
**Results**, which contains **Log** and **Result files** views. Tab switches keep
parameters and the selected script. Selected paths show in their fields, with the
full path available on hover. Narrow windows place the script list above the tabs.
**Parameters** uses graphical controls to configure workflow inputs, CPU cores
and variable overrides.
The optional C++ runner reuses BaneWfn's parser, module expansion, raw input,
variables and arrays, builtins, text extraction, postprocessing, collection and
citations. **The desktop workflow entry starts one Multiwfn process** and keeps
that same process for GUI operations and subsequent workflow blocks. Its PID is
shown in the panel. Loaded data and modified scientific state remain in memory;
ordinary blocks do not reload the startup file. Selecting a different startup
file or using `wfn_rebase=` explicitly invokes Multiwfn's native reload menu.
Returning to the workbench refreshes its structure and available volumes.
Result files and logs remain in each job directory.

The existing engine-first GUI launch remains available with its previous batch
backend. Use the new launcher for persistent sessions; it does not attach to an
arbitrary terminal process. Persistent sessions support Linux/WSL, Windows and
macOS. Use `run-workflows-gui` on Linux, `Multiwfn.exe` on Windows, or open
`Multiwfn.app` on macOS. The macOS tar package supports
`resources/tools/matterviz-desktop --launch [startup-file]`. Omit the startup
file to choose it in a native dialog. A failed persistent launch reports an error;
it does not silently switch to the batch backend.

Engine startup logs are under the user's temporary directory in
`multiwfn-session-logs`. The adapter uses private pipes and Fortran scratch input
streams, so it does not need to write into the installation or application bundle.

## Build and install

The runner is optional. Ordinary noGUI and GUI builds keep their current language
and runtime requirements when `MULTIWFN_WITH_WORKFLOWS=OFF`, the default.
For a complete Linux desktop build with the frontend and Rust host:

```sh
bash tools/build_workflows_gui.sh
./build/run-workflows-gui /absolute/path/to/startup.fchk
```

Omit the startup argument to open the native file chooser. The launcher's engine,
host and runner are all under `build/`; an HTML file alone is not the application.
The helper uses existing toolchains and dependencies; normal Linux requirements
include GTK3/WebKitGTK 4.1 development packages. This WSL development build also
supports extracted Ubuntu dependencies under `build/gui-sysroot`. When the system
has no WebKit helpers, `bwrap` exposes those local helpers in a private mount
namespace without installing files into `/usr`. This local runtime is specific
to the Ubuntu 24.04 x86-64 build, not a universal Linux bundle.
When GTK3 development files are available, CMake also stages the small
`matterviz-file-dialog` helper. Linux file/folder selection uses this separate
process so WSL does not need an XDG desktop portal and GTK thread ownership does
not conflict with Tauri. Packages without the helper retain the existing portal
picker. The helper also ships in the optional workflows install component.

To enable just the C++17 runner in an existing build:

```sh
cmake -S . -B build-matterviz-gui -DMULTIWFN_GUI_BACKEND=matterviz \
  -DMULTIWFN_WITH_WORKFLOWS=ON -DCMAKE_BUILD_TYPE=Release
cmake --build build-matterviz-gui --parallel 4
```

The frontend must be rebuilt and the existing Rust host must be rebuilt as
described in `docs/build.md`. The workflow feature introduces no Python, Qt,
JavaScript server or additional Rust package at runtime. The optional runner uses
the C++ standard library. Python3 is additionally needed at configure time for
the session entry generator on all three platforms, not at runtime. Existing native file dialogs
select local paths.

The default staging layout is:

```text
Multiwfn_MatterVizGUI
run-workflows-gui
settings.ini
resources/
  tools/matterviz-desktop
  tools/multiwfn-workflow
  workflows/conf/*.conf
  workflows/scripts/*.bw
  workflows/scripts/*.bwc
  workflows/LICENSE
  workflows/NOTICE.md
```

The host discovers the runner beside itself. It resolves the executable of the
Multiwfn process that launched the desktop host from the launcher's process ID,
and locks workflows to that exact executable. There is no engine picker or
fallback to a different packaged build. Without the optional runner, the panel
explains that it must be installed. If the current executable cannot be resolved,
workflow execution stays unavailable and the panel reports that error.

To build the runner alone without a Fortran compiler or math libraries:

```sh
cmake -S frontend/workflows -B build/workflow-runner -DCMAKE_BUILD_TYPE=Release
cmake --build build/workflow-runner --parallel 4
cmake --install build/workflow-runner --component workflows --prefix /chosen/prefix
```

This installs into `bin/resources/tools` and `bin/resources/workflows`. Keep the
runner and workflow resources together when relocating them. The Actions packages
include these resources, and the macOS app packager stages them when supplied
its workflow options. Ordinary builds without workflows retain their existing
engine-first launch behavior.

For source-tree development, the host supports explicit absolute paths through
`MULTIWFN_WORKFLOW_RUNNER`, `MULTIWFN_WORKFLOW_LIBRARY`,
`MULTIWFN_WORKFLOW_CONF`. These select local
resources; they are not web request parameters.

## Run a workflow

1. Open **Workflows**. Use the bundled density/Mayer examples or choose a script
   folder. Select a script to review its source.
2. **Startup file** initially uses the file loaded by the current Multiwfn
   session. Choosing another input explicitly reloads the current engine.
3. **Results folder** defaults to `<input directory>/<input stem>_workflows`.
   It follows startup-file changes until a results folder is chosen explicitly;
   an explicit choice is preserved. The automatic directory is created only
   when preflight or execution starts. Set CPU cores
   and any variable overrides, one `key=value` per line. For example, `grid=1`
   selects the bundled density workflow's coarse grid.
4. Click **Preflight** to expand the script and generate menu-input files without
   starting Multiwfn or executing `%command`. Review the log and generated files.
5. Click **Run workflow**. Each click creates a new `workflow-<time>-<sequence>`
   directory. In a persistent session, **Cancel workflow** waits for the active
   block to finish and return to the main menu, then skips remaining blocks.
   It preserves the engine and completed output. Legacy batch cancellation
   retains its process-tree termination behavior.

Closing the workflow panel leaves an active job running. Returning from or
closing the workbench requests cancellation, waits for the current block, and
then closes the owned engine. One workflow may run per workbench at a time.
The core count defaults to one and can be changed explicitly.

The runner uses batch `end`. It rejects terminal `wait`, debug pauses and
interactive builtins before handing control to a terminal. Unresolved `?`
variables are rejected in session mode; supply their values in the GUI overrides.
Platform shells and any tools used by `%command` must be available separately.

Session blocks must finish at Multiwfn's main menu. Only the final generated
`q` in module/builtin expansion is removed. Literal `%raw` input is preserved;
main-menu `q`/`-10` fails the workflow while keeping the engine alive. Automatic
blocks suppress interactive GUI openings. A script that stops in a submenu or
requests unexpected input cannot reach a cancellation boundary; this first
version provides no manual input console or recovery from that condition.

## Inputs, logs and results

The selected script's folder and companion files are copied into the fresh job
directory. Relative paths within that folder keep working. A wavefunction chosen
as the startup file is passed by absolute path and is not copied separately. Keep large wavefunction
collections outside the script library. Companion inputs are bounded to 64 MiB,
1024 files and eight directory levels; symlinks are rejected, and hidden files,
`build`, `target` and `node_modules` directories are excluded.
Result directories marked with `.workflow-results.json` are excluded from script
discovery and input staging, including earlier runs for other startup files.
Results may be a child of the script folder, but cannot be the script folder
itself. This keeps automatic results beside inputs without recursively copying
generated workflows and their outputs.

Bundled modules are copied into `.workflow-conf`, then adjacent `.conf` files and
`conf/*.conf` from the script folder override them. Inline `.bwc` configurations
retain BaneWfn's precedence. A job-specific `banewfn.rc` selects the requested
engine, module directory, cores and `references.bib`; a script folder's rc is not
used to redirect execution. An adjacent engine `settings.ini` is copied when
available. References and module citations retain BaneWfn's existing behavior.

`workflow-run.json` records the runner, engine, original script, wavefunction,
resources and variable overrides. `workflow.log` holds runner stdout/stderr, and
Multiwfn keeps its normal per-task `.out` files so `%grep` and `${output}` continue
to work. The panel shows a bounded log tail, including the most recent `.out`.
`workflow-status.json` records terminal state and exit code. Existing results are
never reused or removed.

**Preflight complete** indicates successful script expansion. **Completed**
indicates that the runner and the workflow's required checks returned success.
Scientific correctness still depends on the workflow and its expected artifacts;
use required `%grep` checks when the presence of a result alone is insufficient.

## Review scope

Related discussion: [BaneWfn stdin control, issue #58](https://github.com/Stardust0831/Multiwfn/issues/58),
[GUI integration principles, PR #27](https://github.com/Stardust0831/Multiwfn/pull/27)
and [native host architecture, PR #26](https://github.com/Stardust0831/Multiwfn/pull/26).
The contribution is limited to frontend, host orchestration, optional build
integration, tests and documentation. Root Fortran, `ext/` and `libreta_hybrid/`
stay unchanged. The optional workflow build generates a copy of `Multiwfn.f90`
with two entry/input substitutions; unique-anchor checks fail configuration if
upstream changes those locations. The adapter schedules complete menu-input blocks
and redirects existing output, without replacing scientific routines.

Suggested issue/PR title: **Run BaneWfn workflows in an owned Multiwfn GUI session**.
The review description should explain the private control channel, generated
entry, retained memory, explicit reload and block-boundary cancellation, optional
dependencies, retained licenses and validation results in `EDIT_STATUS.md`.
No GitHub issue or PR has been posted by this implementation task.

The pinned BaneWfn origin and local adapter changes are recorded in
`frontend/workflows/NOTICE.md`. Redistribute its Apache license with the runner
and preserve this repository's Multiwfn license and attribution separately.
