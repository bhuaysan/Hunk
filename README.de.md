# Hunk

Hunk ist eine moderne, lokale Desktop-Werkbank zum Erstellen und Verwalten von CHD-Abbildern für Emulatoren. Linux ist die erste unterstützte Plattform. Die Anwendung befindet sich in aktiver Entwicklung und führt noch keine Konvertierungen aus.

[English version](README.md)

## Aktueller Stand

Das Repository-Grundgerüst ist fertig: Hunk besitzt eine startbare Tauri-2-Anwendung mit Svelte 5 und TypeScript im Frontend, einem Rust-Backend, Projektprüfungen und Continuous Integration. Als Nächstes folgt die Quelldateierkennung.

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

## Lizenz

Hunk ist freie Software unter der [GNU General Public License, Version 3 oder neuer](LICENSE).
