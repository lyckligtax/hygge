# Token
Als Token nutzen wir JWT um einen eingeloggten Nutzer zu authentifizieren.
Jedes Device hat einen eigenen Token.
## Payload
- [[User Management#Account|Internal Id]]
- [[User Management#User|User Id]]
- [[User Management#Tenant|Tenant Id]]
- [[Ids#Intern|Token Id]]
# Validierung
1. prüfen ob der Token selbst valide und nicht abgelaufen ist
2. prüfen ob die Token Id invalidiert wurde
3. prüfen ob die Tokens der [[User Management#Account|Internal Id]] nach Erstellung des aktuellen Tokens invalidiert wurden
# Invalidierung
JWT ist zustandlos und auch nach Logout oder Passwordwechsel eines Nutzers gültig. Um einen Token zu invalidieren wird eine Blacklist von Tokens geführt welche bei jedem API Aufruf geprüft werden muss.

Vorteil gegenüber auf dem Server gespeicherten Session Tokens:
- Blacklist ist potentiell kleiner und somit schneller abfragbar
- notwendige Daten können bereits im Token hinterlegt werden
- abgelaufene Tokens können direkt abgelehnt werden ohne die Blacklist zubenutzen
## Invalidierung eines Tokens
Die [[Ids#Intern|Token Id]] muss der Blacklist hinzugefügt werden
## Invalidierung alle Tokens eines Nutzers
Die [[User Management#Account|Internal Id]] muss mit samt eines Zeitstempels der Blacklist hinzugefügt werden
# Aktionen
## Login
Nutzer loggt sich mittels [[Ids#Extern|externer Id]] und Passwort ein und erhält dafür einen Token
## Logout
Der zur zeit genutze Token wird invalidiert
## User wechsel
Ein Account kann mehrere Nutzer haben. Wechselt ein Account von einem Nutzer in den anderen wird der bisherige Token invalidiert und ein neuer generiert.
## Passwort wechsel /Account löschen
Alle Tokens des eingeloggten Accounts müssen invalidiert werden. Da wir aber nicht wissen was für Token existieren (nicht auf dem Server gespeichert) müssen wir alle Tokens des Accounts invalidieren.
Dies machen wir indem wir die [[User Management#Account|Internal Id]] und den aktuellen Zeitstempel in der Blacklist speichern.