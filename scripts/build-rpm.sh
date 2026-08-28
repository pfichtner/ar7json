#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "Usage: $0 <target> <version> <workspace>" >&2
    exit 1
fi

TARGET="$1"
VERSION="$2"
WORKSPACE="$3"

VERSION="${VERSION#v}"

case "$TARGET" in
    x86_64-unknown-linux-gnu)  RPM_ARCH=x86_64 ;;
    aarch64-unknown-linux-gnu) RPM_ARCH=aarch64 ;;
    *)
        echo "Error: Unsupported target for RPM package: $TARGET" >&2
        exit 1
        ;;
esac

RPM_DIR=$(mktemp -d)
trap 'rm -rf "$RPM_DIR"' EXIT

mkdir -p "$RPM_DIR"/{BUILD,SOURCES,SPECS,SRPMS}
mkdir -p "$RPM_DIR/RPMS/noarch"

cp "target/${TARGET}/release/ar7json" "$RPM_DIR/SOURCES/"
cp "${WORKSPACE}/generated/completions/ar7json.bash" "$RPM_DIR/SOURCES/ar7json.bash"
cp "${WORKSPACE}/generated/completions/_ar7json" "$RPM_DIR/SOURCES/_ar7json"
cp "${WORKSPACE}/generated/completions/ar7json.fish" "$RPM_DIR/SOURCES/ar7json.fish"
cp "${WORKSPACE}/generated/man/ar7json.1" "$RPM_DIR/SOURCES/ar7json.1"
cp "${WORKSPACE}/generated/symlinks/symlinks" "$RPM_DIR/SOURCES/symlinks"

cat > "$RPM_DIR/SPECS/ar7json.spec" <<EOF
Name:           ar7json
Version:        $VERSION
Release:        1
Summary:        Standalone AR7 to JSON converter for AVM FRITZ!Box ar7.cfg configuration files
License:        MIT
URL:            https://github.com/pfichtner/ar7json
%description
Standalone converter between AVM FRITZ!Box ar7.cfg (AR7) format and JSON.
%install
mkdir -p %{buildroot}/usr/bin
cp %{_sourcedir}/ar7json %{buildroot}/usr/bin/ar7json
cd %{buildroot}/usr/bin
while read name; do ln -s ar7json "\$name"; done < %{_sourcedir}/symlinks
mkdir -p %{buildroot}/usr/share/bash-completion/completions
mkdir -p %{buildroot}/usr/share/zsh/vendor-completions
mkdir -p %{buildroot}/usr/share/fish/vendor_completions.d
cp %{_sourcedir}/ar7json.bash %{buildroot}/usr/share/bash-completion/completions/ar7json
cp %{_sourcedir}/_ar7json %{buildroot}/usr/share/zsh/vendor-completions/_ar7json
cp %{_sourcedir}/ar7json.fish %{buildroot}/usr/share/fish/vendor_completions.d/ar7json.fish
mkdir -p %{buildroot}/usr/share/man/man1
cp %{_sourcedir}/ar7json.1 %{buildroot}/usr/share/man/man1/ar7json.1
%files
%attr(755,root,root) /usr/bin/ar7json
/usr/bin/ar7-to-json
/usr/bin/json-to-ar7
/usr/bin/ar7-check
/usr/bin/ar7-fmt
/usr/share/bash-completion/completions/ar7json
/usr/share/zsh/vendor-completions/_ar7json
/usr/share/fish/vendor_completions.d/ar7json.fish
/usr/share/man/man1/ar7json.1
EOF

rpmbuild -bb --define "_topdir $RPM_DIR" --target "$RPM_ARCH" "$RPM_DIR/SPECS/ar7json.spec"
cp "$RPM_DIR/RPMS/$RPM_ARCH"/ar7json-*.rpm "ar7json-${VERSION}-${TARGET}.rpm"
