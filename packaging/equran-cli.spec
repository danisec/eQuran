Name:           equran-cli
Version:        1.1.0
Release:        1%{?dist}
Summary:        Al-Quran CLI player with qari audio and TTS translation
License:        MIT
URL:            https://github.com/danisec/eQuran
Source0:        %{name}-%{version}.tar.gz
BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  alsa-lib-devel
Recommends:     python3
Recommends:     python3-pip
%description
eQuran plays Quran recitation from EQuran.id with selectable qari and
text-to-speech translation in Indonesian or English. The translation voice runs
on-device through Supertonic 3; recitation needs no Python runtime at all.
%prep
%autosetup
%build
cargo build --release --locked --features audio
%install
install -Dm755 target/release/equran-cli %{buildroot}%{_bindir}/equran-cli
%files
%license LICENSE
%doc README.md
%doc DEVELOPER.md
%doc tts/requirements.txt
%doc tts/setup.sh
%doc tts/setup-gpu.sh
%doc tts/tts_supertonic.py
%{_bindir}/equran-cli
%changelog
* Fri Nov 14 2026 eQuran Maintainers <maintainers@example.invalid> - 1.1.0-1
- Track the 1.1.0 release; drop the unused espeak-ng dependency
