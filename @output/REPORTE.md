# Validación Post-Refactor — rust_scraper

**Fecha:** 2026-06-29
**Refactor:** `crawler_service.rs` → `crawler/{collector,discovery,engine}` (1300→107 líneas)
**Commits:** PRs #69, #72, #73 mergeados a `main`

---

## Resumen

| Sitio | Estado | Resultado |
|-------|--------|-----------|
| Books to Scrape | ✅ | Parseo perfecto, 73 URLs descubiertas, contenido extraído |
| Web Scraper Test (e-commerce) | ✅ | Parseo productos OK, límite: paginación JS no capturada |
| HackerRank | ✅ | SSR suficiente, contenido extraído (React SPA) |
| eBay | ✅ | Sin WAF, navegación extraída, contenido limitado por JS |
| Amazon | ❌ | AWS WAF bloquea — challenge JS requerido |

---

## Test 1: Books to Scrape — Parseo HTML básico

**URL:** `https://books.toscrape.com/`
**Comando:** `--max-pages 5 --max-depth 2`

- ✅ 73 URLs descubiertas del HTML estático
- ✅ 5/5 páginas scrapeadas sin errores
- ✅ Readability extrajo: títulos, precios (£), imágenes, enlaces a detalle
- ✅ Export JSONL generado
- **Output:** `@output/books-to-scrape/`

**Veredicto:** 🔥 Sólido. El scraper maneja catálogos HTML estructurados sin esfuerzo.

---

## Test 2: Web Scraper Test — Navegación y paginación

**URL:** `https://webscraper.io/test-sites/e-commerce/allinone`
**Comando:** `--max-pages 10 --max-depth 2`

- ✅ Landing page parseada: productos, precios, descripciones
- ✅ 23 URLs descubiertas
- ⚠️ Paginación con JS ("Load more") no capturada
- ⚠️ Readability falló en subdominios (forum, cloud) — esperado, páginas sin artículo
- **Output:** `@output/web-scraper-test/`

**Veredicto:** ✅ Bueno para contenido estático. JS dinámico es limitación conocida (feature flag `--force-js-render` no implementado). Product detail pages no se scrabearon por ser subdominio externo.

---

## Test 3: HackerRank — JavaScript + Auth

**URL:** `https://www.hackerrank.com/`
**Comando:** `--single-page`

- ✅ Página renderizada correctamente por SSR
- ✅ Readability extrajo: tagline, descripciones de productos, CTAs
- ✅ Sin WAF, sin bloqueos
- ⚠️ Contenido dinámico (React SPA) no disponible sin JS runtime
- **Output:** `@output/hackerrank/`

**Veredicto:** ✅ Aceptable. El SSR de HackerRank permite extracción útil. Para contenido post-login se necesitaría un navegador headless.

---

## Test 4: Amazon / eBay — WAF y anti-bot

### Amazon
**URL:** `https://www.amazon.com/`
**Comando:** `--single-page`

- ❌ **AWS WAF bloqueó** — challenge de token cifrado (`AwsWafIntegration`)
- Readability falló (no hay contenido que extraer)
- Sin UA rotation exitosa (flag `--user-agent` no existe en CLI)
- **Output:** `@output/amazon-ebay/amazon.com/index.md` (12 líneas, solo script WAF)

**Veredicto:** ❌ No superable sin JS runtime o rotación de fingerprints.

### eBay
**URL:** `https://www.ebay.com/`

- ✅ Sin WAF
- ✅ Readability extrajo navegación principal
- ⚠️ Contenido dinámico limitado (JS-heavy)
- **Output:** `@output/amazon-ebay/ebay.com/index.md` (20 líneas)

**Veredicto:** ⚠️ Accesible pero contenido superficial sin headless browser.

---

## Hallazgos Clave

### 1. WAF Detection existe pero no salva el resultado
El código tiene `ScraperError::WafBlocked` y lógica de detección, pero Amazon devuelve HTML 200 con challenge WAF embedido en scripts. El scraper no lo detecta como WAF y el output termina siendo basura. **Fix:** Reforzar detección de WAF por patrones de script (`awsWafCookieDomainList`, `gokuProps`).

### 2. Sin `--user-agent` en CLI
La struct `Args` no expone `--user-agent`, pero `NetworkOptions.user_agent` existe. **Fix:** Añadir flag CLI para permitir UA custom.

### 3. Paginación JS no capturable
`--force-js-render` existe como flag pero no implementado. Para sitios con infinite scroll / "Load more" se necesita headless browser (Firecrawl, Playwright).

### 4. Discovery cruza subdominios
Web Scraper Test descubrió enlaces a `forum.webscraper.io`, `cloud.webscraper.io` y los scrapeó (fallando Readability). **Mejora:** Permitir filtrar por mismo subdominio.

---

## Outputs Generados

```
@output/
├── books-to-scrape/          ✅ Parseo perfecto
│   ├── books.toscrape.com/
│   │   ├── index.html.md
│   │   └── catalogue/...
│   └── export.jsonl
├── web-scraper-test/         ✅ Productos OK, JS limitado
│   ├── webscraper.io/
│   └── export.jsonl
├── hackerrank/               ✅ SSR suficiente
│   ├── hackerrank.com/
│   │   └── index.md
│   └── export.jsonl
├── amazon-ebay/              ⚠️ eBay OK, Amazon WAF
│   ├── amazon.com/           ❌ (WAF block)
│   ├── ebay.com/             ✅ (navegación)
│   └── export.jsonl
└── REPORTE.md
```
