# Trazabilidad de ingeniería y referencias

## Regla

`FUENTE → REQUISITO TATACOA → DECISIÓN → IMPLEMENTACIÓN → PRUEBA → EVIDENCIA QA`

Esta matriz orienta trazabilidad. No afirma conformidad integral, certificación ni aplicabilidad universal de ninguna fuente.

| Ref | Fuente oficial | Relevancia TATACOA | Requisito/decisión | Estado |
|---|---|---|---|---|
| REF-NIST-115 | NIST SP 800-115 | testing/assessment, planificación y registro de pruebas | conservar contexto y actividad de evaluación | BASELINE |
| REF-NIST-SSDF | NIST SP 800-218 v1.1 | desarrollo seguro | integrar prácticas secure development al SDLC | BASELINE |
| REF-ISO-27037 | ISO/IEC 27037:2012 | identificación, recolección, adquisición y preservación de evidencia digital | preservación y tratamiento explícito de originales | BASELINE |
| REF-ISO-27041 | ISO/IEC 27041:2015 | métodos fit-for-purpose y evidencia de validación | requisitos verificables y QA demostrable | BASELINE |
| REF-ISO-27042 | ISO/IEC 27042:2015 | continuidad, validez, reproducibilidad, repetibilidad y revisión independiente | replay, provenance y verifier independiente | BASELINE |
| REF-OWASP-WSTG | OWASP WSTG | metodología de pruebas web y documentación | conocimiento, evidencia técnica y reproducción cuando aplique | BASELINE |
| REF-COL-MSPI | MinTIC Resolución 02277 de 2025 / Anexo MSPI | contexto colombiano de seguridad/privacidad cuando resulte aplicable | mappings trazables sin afirmar obligatoriedad universal | BASELINE |
| REF-RFC3161 | RFC 3161 + actualización RFC 5816 | trusted timestamp / integridad temporal | investigación/diseño activo; mecanismo TATACOA aún no aprobado | ACTIVE-RESEARCH |

## Fuentes oficiales

- NIST SP 800-115: https://csrc.nist.gov/pubs/sp/800/115/final
- NIST SP 800-218 v1.1: https://csrc.nist.gov/pubs/sp/800/218/final
- ISO/IEC 27037:2012: https://www.iso.org/standard/44381.html
- ISO/IEC 27041:2015: https://www.iso.org/standard/44405.html
- ISO/IEC 27042:2015: https://www.iso.org/standard/44406.html
- OWASP WSTG: https://wstg.owasp.org/
- MinTIC — Normatividad de Gobierno Digital: https://gobiernodigital.mintic.gov.co/portal/Politica-de-Gobierno-Digital/Normatividad/
- RFC 3161: https://www.rfc-editor.org/info/rfc3161/
- RFC 5816: https://www.rfc-editor.org/info/rfc5816/

## Notas de vigencia verificadas en septiembre de 2026

- NIST publica SP 800-115 como Final.
- NIST publica SSDF v1.1 (SP 800-218) como Final.
- ISO mantiene las fichas oficiales citadas para ISO/IEC 27037:2012, 27041:2015 y 27042:2015; la vigencia/aplicabilidad debe volver a comprobarse cuando una decisión dependa de ellas.
- MinTIC lista oficialmente la Resolución 02277 de 2025 y su Anexo 1 MSPI; el mapping exacto debe verificarse contra el texto normativo antes de convertirlo en requisito.
- RFC 3161 continúa publicado y RFC Editor indica que fue actualizado por RFC 5816.

## Regla para nuevas decisiones

Antes de convertir una referencia en requisito normativo obligatorio para un engagement concreto se debe verificar aplicabilidad, versión, texto fuente y jurisdicción.

Cuando una capacidad pase de HORIZON/RESEARCH a diseño activo, sus fuentes oficiales y decisiones deben actualizarse aquí antes de afirmar implementación o cumplimiento.
