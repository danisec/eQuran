Name:           equran-cli
Version:        0.1.0
Release:        1%{?dist}
Summary:        Al-Quran CLI player with qari audio and TTS translation

License:        MIT
URL:            https://equran.id
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
Requires:       mpv
Requires:       espeak-ng
Recommends:     python3
Recommends:     python3-pip

%description
EQuran CLI plays Quran recitation from EQuran.id with selectable qari and
text-to-speech translation in Indonesian or English.

%prep
%autosetup

%build
cargo build --release --locked --features audio

%install
install -Dm755 target/release/equran-cli %{buildroot}%{_bindir}/equran-cli

%files
%license LICENSE*
%doc README.md
%doc tts/requirements.txt
%doc tts/setup.sh
%doc tts/tts_wibowo.py
%{_bindir}/equran-cli

%changelog
* Fri May 22 2026 EQuran CLI Maintainers <maintainers@example.invalid> - 0.1.0-1
- Initial Fedora RPM package
