# Hunk

Hunk ist eine moderne, lokale Desktop-Werkbank zum Erstellen und Verwalten von CHD-Abbildern für Emulatoren. Linux ist die erste unterstützte Plattform. Die Anwendung befindet sich in aktiver Entwicklung.

[English version](README.md)

## Aktueller Stand

Das Repository-Grundgerüst, die Quelldateierkennung, die gepinnte `chdman`-Integration, die dauerhafte verifizierte Job-Engine, die responsive lokalisierte Workbench, die Barrierefreiheitsprüfung und die Ende-zu-Ende-Validierung sind fertig. Hunk erkennt CUE/BIN-, GDI-, ISO- und CHD-Eingaben rekursiv, fasst referenzierte Trackdateien zu Quellsätzen zusammen, verarbeitet sie in einer kollisionssicheren seriellen Warteschlange und bewahrt die letzten 100 Job-Einträge lokal auf. Als Nächstes folgt die Linux-Paketierung.

Der festgelegte Umfang und das Sicherheitsmodell stehen im [Implementierungsplan](docs/IMPLEMENTATION_PLAN.md). Die kompakte öffentliche Planung steht in der [Roadmap](ROADMAP.md).

## Entwicklung

Installiere die [Voraussetzungen für Tauri 2](https://v2.tauri.app/start/prerequisites/) unter Linux sowie Rust, Node.js 22 und pnpm 11. Danach kann Hunk so gestartet werden:

```sh
pnpm install
pnpm tauri dev
```

Wichtige Prüfungen:

```sh
pnpm format:check
pnpm check
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Disc-Abbilder, Inhalte des lokalen `Test/`-Verzeichnisses, erzeugte `chdman`-Binärdateien, Pakete, Anwendungsdaten und Zugangsdaten dürfen nie in das Repository aufgenommen werden. Die vollständigen Arbeitsregeln stehen in [AGENTS.md](AGENTS.md).

### Ende-zu-Ende-Validierung

Die regulären Rust-Tests erzeugen kleine deterministische CD-/DVD-Fixtures in temporären
Verzeichnissen und decken mit einem steuerbaren Sidecar-Testprozess erfolgreiche Roundtrips und
Sicherheitsfehler ab. Der optionale Test mit dem echten Sidecar ist in
[docs/CHDMAN.md](docs/CHDMAN.md) beschrieben.

Nach dem Bau des freigegebenen Sidecars können die ignorierten lokalen Daten in `Test/` ausdrücklich
geprüft werden:

```sh
./scripts/test-local-media.sh
```

Der Harness erwartet genau die drei im Implementierungsplan beschriebenen repräsentativen
Quellsätze, schreibt sämtliche Ausgaben in temporären Speicher und bestätigt, dass alle Quelldateien
unverändert bleiben.

## Lizenz

Hunk ist freie Software unter der [GNU General Public License, Version 3 oder neuer](LICENSE).
