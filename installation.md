## Installation

1. exe in einem Ordner speichern
2. config Vorlage in diesen kopieren und anpassen
3. In der Windows startleiste nach "Umgebungsvariablen" suchen, diese öffnen, bei "Erweitert" auf "Umgebungsvariablen"
   klicken
4. Neue Systemvariable erstellen:
    - Name: `EM_CONFIG
    - Wert: Pfad zur config Datei (kopiert aus der Adressleiste des Explorers plus Dateinamen,
      e.g. `C:\Users\user\Documents\emergency-mail\config.yaml`)
    - Ok klicken
5. Rechtsklick auf die exe und "Verknüpfung erstellen"
6. `Windows` + `R` drücken, `shell:startup` eingeben und mit Enter bestätigen
7. Verknüpfung in den Ordner ziehen, der sich öffnet
8. evtl. die Verknüpfung umbenennen ("- Shortcut" entfernen)
9. exe starten (einmalig, da sie nur bei Systemstart startet)
10. Fertig

## Deinstallation

1. Verknüpfung aus dem Autostart Ordner löschen (`Windows` + `R`, `shell:startup`, Enter)
2. exe löschen
3. Umgebungsvariablen löschen (siehe schritt 3 und 4):
    - `EM_CONFIG` auswählen
    - löschen klicken

- Fertig