#!/usr/bin/env bash
# Run beside Multiwfn after moving the unpacked application to its final location.
set -euo pipefail
app_root="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
if [[ $# == 0 ]]; then
  output_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
elif [[ $# == 2 && $1 == --output-dir ]]; then
  output_dir="$2"
else
  echo "Usage: $0 [--output-dir DIRECTORY]" >&2
  exit 2
fi
for executable in Multiwfn run-workflows-gui; do
  [[ -x "$app_root/$executable" ]] || { echo "Missing $app_root/$executable" >&2; exit 1; }
done
[[ -f "$app_root/resources/multiwfn.png" ]] || { echo 'Missing application icon' >&2; exit 1; }
# Desktop Entry values and quoted Exec arguments have separate escape layers.
# env keeps literal '%' in installation paths out of GIO's executable lookup
# (which happens before desktop field-code expansion).
# https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html
case "$app_root" in
  *$'\n'*|*$'\r'*|*$'\t'*|*=*) echo 'Unsupported application directory for a desktop entry' >&2; exit 1 ;;
esac
exec_path="$app_root/Multiwfn"
exec_path="${exec_path//\\/\\\\}"
exec_path="${exec_path//\"/\\\"}"
exec_path="${exec_path//\$/\\\$}"
exec_path="${exec_path//\`/\\\`}"
exec_path="${exec_path//%/%%}"
exec_path="${exec_path//\\/\\\\}"
icon_path="${app_root//\\/\\\\}/resources/multiwfn.png"
mkdir -p -- "$output_dir"
entry="$(mktemp "$output_dir/.multiwfn-desktop.XXXXXXXX")"
trap 'rm -f -- "$entry"' EXIT
cat > "$entry" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=Multiwfn
Comment=Wavefunction analysis and BaneWfn workflows
Comment[zh_CN]=波函数分析与 BaneWfn 工作流
Exec=env "$exec_path" %f
Icon=$icon_path
Terminal=false
Categories=Education;Science;Chemistry;
Keywords=Multiwfn;BaneWfn;MatterViz;
StartupNotify=false
EOF
chmod 755 "$entry"
mv -f -- "$entry" "$output_dir/Multiwfn-Workflows.desktop"
trap - EXIT
printf 'Application entry: %s/Multiwfn-Workflows.desktop\n' "$output_dir"
if [[ -x "$app_root/banewfn" ]]; then
  workflow_exec="${exec_path%Multiwfn}banewfn"
  cat > "$output_dir/BaneWfn.desktop" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=BaneWfn
Comment=Configure and run wavefunction analysis workflows
Comment[zh_CN]=配置与运行波函数分析工作流
Exec=env "$workflow_exec" %f
Icon=$icon_path
Terminal=false
Categories=Education;Science;Chemistry;
StartupNotify=false
EOF
  chmod 755 "$output_dir/BaneWfn.desktop"
fi
