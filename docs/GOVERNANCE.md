# Gobierno del repositorio

## Objetivo

Definir las reglas de decisión y control que rigen el desarrollo de T·A·T·A·C·O·A antes de incorporar arquitectura o código sustancial.

## Jerarquía de decisiones

Para el trabajo cotidiano se aplica:

1. decisión explícita de la Validación Humana;
2. políticas de seguridad y gobierno del repositorio;
3. ADR y especificaciones aprobadas;
4. documentación vigente del producto;
5. implementación y pruebas;
6. propuestas o borradores.

El código existente no convierte automáticamente una decisión accidental en requisito.

## Protección de `main`

`main` representa el estado integrado aprobado. El desarrollo se realiza en ramas. Ningún agente debe hacer merge a `main` sin autorización explícita de la Validación Humana.

Cuando las capacidades de GitHub lo permitan, estas reglas de proceso deberán reforzarse con controles técnicos de repositorio.

## Cambios que requieren decisión humana explícita

- arquitectura fundamental;
- primitivas o parámetros criptográficos;
- cambios en invariantes de seguridad;
- reducción de controles;
- incorporación de telemetría o servicios externos;
- cambios de licencia o visibilidad;
- publicación de releases;
- compatibilidad declarada;
- cambios que afecten evidencia RAW o procedencia;
- eliminación o migración destructiva de datos;
- cambios de alcance del producto.

## Documentación como control

Las decisiones relevantes deben quedar versionadas. Un ADR registra decisiones arquitectónicas significativas y su contexto; no debe utilizarse para documentar cada cambio trivial.

La documentación distingue claramente entre estado implementado, diseño aprobado, investigación, hipótesis y roadmap.

## Secure development

El proyecto adopta como referencias de diseño prácticas de desarrollo seguro y gestión de vulnerabilidades, sin afirmar cumplimiento automático. La implementación y el QA deben demostrar los controles que el proyecto declare.

## Revisión

Estas reglas pueden evolucionar. Un cambio de gobierno debe ser deliberado, revisable y no puede utilizarse retroactivamente para ocultar una desviación previa.
