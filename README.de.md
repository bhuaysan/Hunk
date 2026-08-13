# Hunk

Hunk ist eine lokale Desktop-Werkbank zum Erstellen und Verwalten optischer CHD-Abbilder für
Emulatoren. Sie erkennt vollständige Disc-Quellsätze, bietet nur gültige Aktionen an und verarbeitet
Konvertierungen in einer dauerhaften, kollisionssicheren Warteschlange. Linux x86_64 ist die erste
unterstützte Plattform.

[English version](README.md)

## Veröffentlichungsstatus

Der Quellbaum ist für Hunk 0.1.0 vorbereitet. AppImage- und Flatpak-Pakete erscheinen auf der
[GitHub-Releases-Seite](https://github.com/bhuaysan/Hunk/releases), sobald eine Binärveröffentlichung
bereitsteht. Solange dort kein Paket angeboten wird, kann Hunk mit der folgenden Anleitung aus dem
Quellcode gestartet werden.

## Funktionen

- Dateien und Ordner über native Dialoge oder Drag-and-drop importieren.
- CUE/BIN-, GDI-, ISO- und CHD-Quellsätze rekursiv erkennen, ohne referenzierte Tracks zu duplizieren.
- CD- und DVD-CHDs erstellen und jedes neu erstellte CHD vor der Veröffentlichung vollständig prüfen.
- CD-CHDs nach CUE/BIN und DVD-CHDs nach ISO extrahieren.
- Metadaten untersuchen und bestehende CHDs ohne Änderungen prüfen.
- Arbeiten in einer dauerhaften seriellen Warteschlange pausieren, abbrechen und wiederholen; die
  letzten 100 Verlaufseinträge bleiben gespeichert.
- Die vollständige Oberfläche auf Deutsch oder Englisch mit hellem und dunklem Design sowie
  reduzierten Bewegungen verwenden.

## Sicherheit und Datenschutz

Hunk löscht oder verändert Quellabbilder niemals und überschreibt keine bestehende Ausgabe.
Verändernde Jobs schreiben eine eindeutig benannte temporäre Datei auf das Ziellaufwerk, prüfen neu
erstellte CHDs und veröffentlichen erst nach Erfolg und ohne Ersetzen. Hunk enthält keine Telemetrie
und benötigt zur Laufzeit keinen Netzwerkzugriff. Job-Verlauf und Einstellungen bleiben in einer
lokalen SQLite-Datenbank, die ausschließlich dem Backend gehört.

Das Flatpak benötigt Zugriff auf das Host-Dateisystem, damit Deskriptoren benachbarte Trackdateien
auflösen und Ausgaben neben beliebigen Quellen schreiben können. Der Webview erhält trotzdem weder
Shell- noch allgemeinen Dateisystemzugriff. Die vollständige Grenze beschreiben die
[Architektur](docs/ARCHITECTURE.md) und der
[Linux-Paketierungsleitfaden](docs/LINUX_PACKAGING.md).

## Unterstützte Arbeitsabläufe

| Eingabe          | Verfügbare Aktionen                                       | Standardausgabe                     |
| ---------------- | --------------------------------------------------------- | ----------------------------------- |
| CUE/BIN oder GDI | CD-CHD erstellen                                          | CHD neben dem Deskriptor            |
| ISO              | Nach ausdrücklicher Medienwahl CD- oder DVD-CHD erstellen | CHD neben der ISO                   |
| CD-CHD           | CUE/BIN extrahieren, untersuchen oder prüfen              | Eine BIN und eine CUE neben dem CHD |
| DVD-CHD          | ISO extrahieren, untersuchen oder prüfen                  | ISO neben dem CHD                   |

Die CD-Extraktion kann optional eine BIN pro Track erzeugen. Parent-/Delta-Abbilder, beschreibbare
CHDs, Metadatenänderungen, `verify --fix` und automatisches Aufräumen der Quellen sind in 0.1 bewusst
nicht enthalten.

## Pakete installieren

Lade AppImage oder Flatpak, `SHA256SUMS` und `mame-mame0289-source.tar.gz` aus derselben
Veröffentlichung herunter. Das MAME-Quellarchiv muss bei einer Weitergabe bei den Paketen bleiben.
Prüfe die Downloads in ihrem Verzeichnis:

```sh
sha256sum --check SHA256SUMS
```

AppImage starten:

```sh
chmod +x Hunk_0.1.0_amd64.AppImage
./Hunk_0.1.0_amd64.AppImage
```

Oder das Flatpak-Bundle für den aktuellen Benutzer installieren:

```sh
flatpak --user install Hunk_x86_64.flatpak
flatpak run app.hunk.Hunk
```

Die AppImage-Basis ist Ubuntu 22.04. Das Flatpak verwendet GNOME-Runtime 50 und kann diese bei der
Installation herunterladen.

## Schnelleinstieg für die Entwicklung

Installiere die [Voraussetzungen für Tauri 2](https://v2.tauri.app/start/prerequisites/) unter Linux,
Rust 1.88 oder neuer, Node.js 22 und pnpm 11. Danach:

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

Der eingecheckte Quellcode enthält kein `chdman`. Baue vor Tests echter Konvertierungen das gepinnte
MAME-0.289-Sidecar:

```sh
./scripts/build-chdman.sh
```

Der [Entwicklungsleitfaden](docs/DEVELOPMENT.md) beschreibt Systempakete, Teststufen, generierte
Fixtures und die Paketierung. Disc-Abbilder, lokale `Test/`-Daten, Binärdateien, Pakete,
Anwendungszustand und Zugangsdaten dürfen nie in das Repository aufgenommen werden.

## Projektdokumentation

- [Architektur und Vertrauensgrenzen](docs/ARCHITECTURE.md)
- [Entwicklung und Tests](docs/DEVELOPMENT.md)
- [Freigegebenes `chdman`-Sidecar](docs/CHDMAN.md)
- [Linux-Paketierung](docs/LINUX_PACKAGING.md)
- [Abhängigkeits- und Lizenzaudit](docs/DEPENDENCIES.md)
- [Veröffentlichungsprozess](docs/RELEASING.md)
- [Implementierungsspezifikation](docs/IMPLEMENTATION_PLAN.md)
- [Roadmap](ROADMAP.md)

Beiträge sind willkommen; lies vor einem Pull Request [CONTRIBUTING.md](CONTRIBUTING.md). Melde
Sicherheitslücken privat nach [SECURITY.md](SECURITY.md).

## Lizenz

Hunk ist freie Software unter der [GNU General Public License, Version 3 oder neuer](LICENSE). Das
gebündelte MAME-`chdman`-Sidecar und andere Abhängigkeiten behalten ihre eigenen Lizenzen und
Hinweise; siehe [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
