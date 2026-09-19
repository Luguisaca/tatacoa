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
| REF-OWASP-WSTG | OWASP WSTG | pruebas web, reporting y artifacts reproducibles | evidencia técnica y reproducción con datos sensibles protegidos | BASELINE |
| REF-COL-MSPI | MinTIC Resolución 02277 de 2025 / MSPI | contexto colombiano de seguridad y privacidad; adopta lineamientos ISO/IEC 27001:2022 para sujetos aplicables | investigar mappings y diseñar controles trazables sin afirmar obligatoriedad universal | BASELINE |
| REF-RFC3161 | RFC 3161 | trusted timestamp | post-V1; distinguir hora confiable de hora local | POST-V1 |

## Fuentes oficiales

- NIST SP 800-115: https://csrc.nist.gov/pubs/sp/800/115/final
- NIST SP 800-218 v1.1: https://csrc.nist.gov/pubs/sp/800/218/final
- ISO/IEC 27037:2012: https://www.iso.org/standard/44381.html
- ISO/IEC 27041:2015: https://www.iso.org/standard/44405.html
- ISO/IEC 27042:2015: https://www.iso.org/standard/44406.html
- OWASP WSTG: https://wstg.owasp.org/
- MinTIC MSPI / Resolución 02277 de 2025: https://gobiernodigital.mintic.gov.co/portal/Politica-de-Gobierno-Digital/Normatividad/
- RFC 3161: https://www.rfc-editor.org/info/rfc3161/

## Notas de vigencia verificadas en septiembre de 2026

- NIST publica SP 800-115 como Final.
- NIST publica SSDF v1.1 (SP 800-218) como Final.
- ISO muestra 27037:2012 publicada y bajo ciclo de revisión.
- ISO muestra 27041:2015 publicada y bajo revisión sistemática en 2026.
- ISO muestra 27042:2015 como edición publicada vigente en su ficha.
- MinTIC lista Resolución 02277 de 2025 y su MSPI actualizado; MinTIC indica que adopta lineamientos ISO/IEC 27001:2022.
- RFC 3161 continúa publicado y fue actualizado por RFC 5816.

Antes de convertir una referencia en requisito normativo obligatorio para un engagement concreto se debe verificar aplicabilidad, versión y jurisdicción.
