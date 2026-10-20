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
Der [Speichervertrag](organization-config/storage-v1.md) und der [Adaptervertrag](organization-config/adapter-v1.md)
ergänzen die Datenbankstruktur und die interne SQLx-Anbindung. Sie erteilen keine Benutzerrechte.
Der [Schema-Wartungsvertrag](organization-config/schema-provisioning-v1.md) ergänzt die privilegierte Registrierung leerer Einrichtungsschemas.
Er ist kein öffentlicher API-Vertrag und keine Freigabe von Fachzugriffen.

Der [Dienstisolation-Testvertrag](organization-config/service-isolation-test-v1.md) beschreibt die technischen Proben aus KC-005B.
Seine Loginrollen und erlaubenden Policies gelten ausschließlich im Testcontainer.
