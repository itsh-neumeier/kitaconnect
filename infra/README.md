# Infrastruktur

P0 enthält nur Strukturdateien. Es gibt keine installierbare Umgebung.
`compose/compose.yaml` definiert noch keine Dienste.
`helm/kitaconnect` erzeugt noch keine Kubernetes-Ressourcen.
Portainer verwendet später die geprüfte Compose-Konfiguration aus KC-032.

KC-032 ergänzt Container, Netzgrenzen, Dienstidentitäten, Ressourcenlimits,
Zustandsprüfungen und Secret Stores. KC-033 ergänzt Datensicherung und Wiederherstellung.
P0 erzeugt keine Images, Tags oder Veröffentlichungen.
