# Schnittstellenverträge

Hier liegen künftig OpenAPI-3.1-Verträge je Dienst unter `openapi/<dienst>/`.
Versionierte Ereignisschemata liegen unter `events/<dienst>/`.
P0 enthält keine fachlichen Endpunkte oder Ereignisse.

Ein Dienst MUSS seinen Vertrag vor der Implementierung versionieren.
Vertragsprüfungen MÜSSEN Berechtigungen und Rückwärtskompatibilität abdecken.
KC-002 dokumentiert die bestätigten Grenzen in der [Dienstmatrix](../docs/architecture/KC-002_Dienstgrenzen.md).
Konkrete Endpunkte und Ereignisformate folgen im jeweiligen Implementierungsissue.
Siehe [P0-Plan](../docs/15_P0_Initialisierung.md).

Der [interne Organisationsmodellvertrag v1](organization-config/model-v1.md) ist das erste P1-Arbeitspaket.
Er beschreibt die Rust-Bibliothek und ist kein HTTP-Vertrag. Öffentliche Endpunkte folgen erst mit geprüften Rechten.
