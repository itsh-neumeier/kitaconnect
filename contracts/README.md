# Schnittstellenverträge

Hier liegen künftig OpenAPI-3.1-Verträge je Dienst unter `openapi/<dienst>/`.
Versionierte Ereignisschemata liegen unter `events/<dienst>/`.
P0 enthält keine fachlichen Endpunkte oder Ereignisse.

Ein Dienst MUSS seinen Vertrag vor der Implementierung versionieren.
Vertragsprüfungen MÜSSEN Berechtigungen und Rückwärtskompatibilität abdecken.
Die technischen Festlegungen folgen in KC-002. Siehe [P0-Plan](../docs/15_P0_Initialisierung.md).
