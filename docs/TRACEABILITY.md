# Trazabilidad de ingeniería y referencias

## Regla

`FUENTE → REQUISITO TATACOA → DECISIÓN → IMPLEMENTACIÓN → PRUEBA → EVIDENCIA QA`

Esta matriz inicia la trazabilidad. No afirma conformidad integral con ninguna fuente.

| Ref | Fuente oficial | Relevancia TATACOA | Requisito/decisión inicial | Estado |
|---|---|---|---|---|
| REF-NIST-115 | NIST SP 800-115 | testing/assessment, planificación y registro de pruebas | conservar contexto y actividad de evaluación | BASELINE |
| REF-NIST-SSDF | NIST SP 800-218 v1.1 | desarrollo seguro | integrar prácticas de secure development al SDLC | BASELINE |
| REF-ISO-27037 | ISO/IEC 27037:2012 | identificación, recolección, adquisición y preservación de evidencia digital | preservación y tratamiento explícito de originales | BASELINE |
| REF-ISO-27041 | ISO/IEC 27041:2015 | métodos fit-for-purpose y evidencia de validación | requisitos verificables y QA demostrable | BASELINE |
| REF-ISO-27042 | ISO/IEC 27042:2015 | continuidad, validez, reproducibilidad, repetibilidad y revisión independiente | replay, provenance y verifier independiente | BASELINE |
| REF-OWASP-WSTG | OWASP WSTG | metodología de pruebas web y documentación de resultados | conocimiento, evidencia técnica y reproducción cuando aplique | BASELINE |
| REF-COL-MSPI | MinTIC Resolución 02277 de 2025 / Anexo MSPI | contexto colombiano de seguridad y privacidad para los sujetos a los que resulte aplicable | investigar mappings y diseñar controles trazables sin afirmar obligatoriedad universal | BASELINE |
| REF-RFC3161 | RFC 3161 | trusted timestamp | post-V1; distinguir hora confiable de hora local | POST-V1 |

## Fuentes oficiales

- NIST SP 800-115: https://csrc.nist.gov/pubs/sp/800/115/final
- NIST SP 800-218 v1.1: https://csrc.nist.gov/pubs/sp/800/218/final
- ISO/IEC 27037:2012: https://www.iso.org/standard/44381.html
- ISO/IEC 27041:2015: https://www.iso.org/standard/44405.html
- ISO/IEC 27042:2015: https://www.iso.org/standard/44406.html
- OWASP WSTG: https://wstg.owasp.org/
- MinTIC — Normatividad de Gobierno Digital (Resolución 02277 de 2025 y Anexo MSPI): https://gobiernodigital.mintic.gov.co/portal/Politica-de-Gobierno-Digital/Normatividad/
- RFC 3161: https://www.rfc-editor.org/info/rfc3161/

## Notas de vigencia verificadas en septiembre de 2026

- NIST publica SP 800-115 como Final.
- NIST publica SSDF v1.1 (SP 800-218) como Final.
- ISO mantiene ISO/IEC 27037:2012 publicada; su ficha muestra que permanece vigente y que existe actividad de revisión.
- ISO mantiene ISO/IEC 27041:2015 publicada; su revisión sistemática de 2026 cerró el 3-sep-2026 y la ficha aún la muestra como estándar vigente.
- ISO mantiene ISO/IEC 27042:2015 publicada; su ficha la muestra como estándar vigente y describe continuidad, validez, reproducibilidad, repetibilidad y revisión independiente.
- MinTIC lista oficialmente la Resolución 02277 de 2025 y su Anexo 1 de lineamientos MSPI. El mapping exacto de controles y referencias del Anexo debe verificarse contra el texto normativo antes de convertirlo en requisito.
- RFC 3161 continúa publicado y el RFC Editor indica que fue actualizado por RFC 5816.

Antes de convertir una referencia en requisito normativo obligatorio para un engagement concreto se debe verificar aplicabilidad, versión, texto fuente y jurisdicción.
