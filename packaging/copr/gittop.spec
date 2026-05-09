Name:           gittop
Version:        %{_version}
Release:        1%{?dist}
Summary:        A lightweight desktop client for GitHub notifications
License:        AGPL-3.0-only
URL:            https://github.com/AmarBego/GitTop

# Pre-built binary tarballs from GitHub releases (one per arch, selected in %prep)
Source0:        https://github.com/AmarBego/GitTop/releases/download/v%{version}/gittop-%{version}-linux-gnu-x86_64.tar.gz
Source1:        https://github.com/AmarBego/GitTop/releases/download/v%{version}/gittop-%{version}-linux-gnu-aarch64.tar.gz

# We're packaging a pre-built binary
%global debug_package %{nil}
ExclusiveArch:  x86_64 aarch64

%description
GitTop is a lightweight desktop client for GitHub notifications.
No browser engine required. Pure Rust. Pure performance.

%prep
%setup -q -c -T -n %{name}-%{version}
%ifarch x86_64
tar -xf %{SOURCE0} --strip-components=1
%endif
%ifarch aarch64
tar -xf %{SOURCE1} --strip-components=1
%endif

%build
# Binary is pre-built, nothing to do

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/256x256/apps
mkdir -p %{buildroot}%{_docdir}/%{name}

install -m 755 gittop %{buildroot}%{_bindir}/gittop
install -m 644 gittop.desktop %{buildroot}%{_datadir}/applications/gittop.desktop
install -m 644 gittop.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/gittop.png

%files
%{_bindir}/gittop
%{_datadir}/applications/gittop.desktop
%{_datadir}/icons/hicolor/256x256/apps/gittop.png
%license LICENSE.md
%doc README.txt

%changelog
* Sat May 09 2026 AmarBego <begovicamar@proton.me> - 0.5.1-1
- Initial release of v0.5.1
