# Threat Model

## Estado

**APROBADO como baseline; debe actualizarse con la implementación.**

## Actores y entradas

- auditor/aprendiz;
- Validación Humana/revisor;
- sistema operativo;
- herramienta externa;
- artifact o bundle importado;
- adapter/parser;
- adversario que intenta manipular evidencia o entradas.

## Activos críticos

- evidencia RAW;
- manifests;
- digests;
- relaciones de procedencia;
- contexto y scope;
- secretos;
- recetas de replay;
- futuras claves criptográficas;
- bundles exportados;
- Knowledge Cards cuando contienen contexto sensible.

## Trust boundaries

- usuario → Core;
- Core → sistema operativo/filesystem;
- Execution → herramienta externa;
- output externo → parser/adapter;
- workspace → export;
- bundle → verifier;
- proyecto → revisor externo.

## Amenazas principales

- scope confusion y contaminación entre engagements;
- command/shell injection;
- output o import malicioso;
- path traversal y symlinks;
- sustitución o modificación de RAW;
- procedencia falsa o ciclos;
- fuga de secretos;
- replay contra contexto equivocado;
- privilegios innecesarios;
- adapter malicioso;
- supply-chain compromise;
- borrado o corrupción;
- manipulación del tiempo local;
- identidad/autoria falsamente inferida de un hash;
- redacción defectuosa;
- contenido activo en evidencia;
- resource exhaustion;
- recorder crash;
- version spoofing;
- alucinación o conocimiento envenenado.

## Invariantes

1. toda Execution pertenece a un Engagement/contexto identificable;
2. RAW nunca se reemplaza por una derivación;
3. toda derivación tiene procedencia;
4. alteración detectable no se silencia;
5. captura fallida/parcial/truncada se declara;
6. IA no valida vulnerabilidades ni evidencia;
7. contexto no equivale a autorización;
8. hora local no equivale a trusted timestamp;
9. hash no demuestra autoría;
10. adapter opera con mínima capacidad;
11. Knowledge no debe copiar secretos RAW innecesariamente;
12. redacción nunca destruye el original;
13. exportación respeta política del engagement;
14. contraseña nunca es clave de datos directa;
15. material criptográfico nuevo usa aleatoriedad adecuada;
16. cifrado, firma y timestamp son propiedades distintas;
17. no existe master key del proveedor;
18. fallo criptográfico nunca degrada silenciosamente a plaintext;
19. verifier no ejecuta artifacts;
20. paths del bundle no escapan su raíz;
21. validación humana es necesaria para promoción a Evidence;
22. aislamiento entre engagements debe probarse negativamente.

## Controles esperados

- executable + argv por defecto;
- shell explícito;
- canonicalización y validación de paths;
- tratamiento hostil de imports/parsers;
- streaming y límites de recursos;
- RAW inmutable por política de aplicación;
- placeholders para secretos en replay;
- least privilege;
- lockfiles, auditoría de dependencias y SBOM;
- renderizado seguro;
- tests negativos y fuzzing en superficies de parsing;
- verifier independiente/read-only.

Los controles se consideran garantías solo después de ser implementados y demostrados.
