Name:           gittop
Version:        %{_version}
Release:        1%{?dist}
Summary:        A lightweight desktop client for GitHub notifications
License:        AGPL-3.0-only
URL:            https://github.com/AmarBego/GitTop

# Pre-built binary tarball from GitHub releases
Source0:        https://github.com/AmarBego/GitTop/releases/download/v%{version}/gittop-%{version}-linux-x86_64.tar.gz

# We're packaging a pre-built binary
%global debug_package %{nil}
ExclusiveArch:  x86_64

%description
GitTop is a lightweight desktop client for GitHub notifications.
No browser engine required. Pure Rust. Pure performance.

%prep
%setup -q -n gittop-%{version}-linux-x86_64

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
