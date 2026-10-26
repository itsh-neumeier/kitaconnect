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

Der [Pool- und Wartungstestvertrag](organization-config/pool-maintenance-test-v1.md) ergänzt Wiederverwendung nach Fehlern und Abbruch sowie konkurrierende Wartung.
Er definiert Testhilfen und keine produktive Pool- oder Migrationssteuerung.

Der [Identitäts- und Sitzungsmodellvertrag](identity-access/session-model-v1.md) beginnt KC-006A im Dienst `identity-access`.
Das interne Modell stellt keine Sitzungsnachweise aus und erteilt keine Anmeldung oder Berechtigung.

Der [Sitzungs-Speichervertrag](identity-access/session-storage-v1.md) ergänzt den internen SQLx-Adapter und abgeleitete Sitzungsnachweise.
Die Anwendungsrolle bleibt durch RLS gesperrt. Die HTTP-Anmeldung und ihre offenen Sicherheitsregeln folgen gesondert.

Die [interne Mitgliedschaftsabfrage über HTTP](organization-config/membership-http-v1.md)
und ihr [OpenAPI-Vertrag](openapi/organization-config/membership-v1.json) verbinden den vorhandenen authentisierten Leser mit einer begrenzten internen Route.
Die Antwort enthält ausschließlich Mitgliedschaft und Fristen. Sie erteilt keine Benutzer- oder MFA-Rechte.

Der [begrenzte Mitgliedschaftsverbraucher](identity-access/membership-consumer-v1.md)
beschreibt den tatsächlichen festen HTTP-Client und die strenge Antwortprüfung im Identitätsdienst.
Sein Datenstand enthält keine MFA-Entscheidung. Der vollständige Anmeldeadapter bleibt gesondert.

Die [aktuelle Rollen-/MFA-Grundlage](identity-access/rights-assurance-v1.md)
beschreibt den eigenen atomaren Rollenbestand und die strenge aktuelle Ableitung für KC-008A.
Die Grundlage erteilt keine Fachberechtigung.

Die [zusammengesetzte aktuelle Anmeldegrundlage](identity-access/current-login-source-v1.md)
verbindet tatsächliche Dienstanmeldung, Mitgliedschaft und Rollenquelle. Die drei Versionen bleiben getrennt.

Die [dauerhafte Quellenbindung](identity-access/source-bindings-v1.md) erhält diese Versionen in Vorgängen, Sitzungsprüfung und Rotation.

Der [Prüfvertrag der tatsächlichen Anmeldekette](identity-access/login-chain-test-v1.md)
verbindet Faktor-Bibliotheken und Sitzungsabschluss mit den tatsächlichen aktuellen Dienstquellen.

Die [Browsergrenze des Gateways](gateway/browser-boundary-v1.md) prüft feste Origin-, Header- und Cookie-Profile.
Sie erteilt allein weder Anmeldung noch CSRF- oder Sitzungsfreigabe.

Die [eigene lokale Anmeldekennung](identity-access/login-name-v1.md) ordnet einen genauen Namen einer internen Identität zu.
Die interne Zuordnung ersetzt weder Faktorprüfung noch Mitgliedschaft und veröffentlicht keinen Kontoabfrage-Endpunkt.
