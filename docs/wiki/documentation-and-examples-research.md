# Documentation and Examples — research

# Documentation and Examples — research

This module contains research findings, competitive analysis, and technical proposals related to enhancing the scraper's capabilities, particularly for exporting data in formats compatible with knowledge management systems like Obsidian.

## 1. Obsidian-Compatible Markdown Export Research

This research report details the specifications and conventions for exporting scraped web content into a format that is directly usable and well-integrated within Obsidian, a popular personal knowledge management (PKM) tool.

### 1.1 Obsidian Markdown Specification

"Obsidian-Compatible" primarily refers to adherence to specific Markdown extensions and file structure conventions that Obsidian natively supports or commonly uses.

#### 1.1.1 YAML Frontmatter (Properties)

Obsidian utilizes YAML frontmatter at the beginning of `.md` files, referred to as "Properties." This is a critical component for metadata.

```yaml
---
title: "Article Title"
tags:
  - web-clip
  - technology
aliases:
  - "Alternative Title"
created: 2026-04-03T17:00:00
modified: 2026-04-03T17:00:00
source: "https://example.com/article"
author: "John Doe"
description: "Page excerpt or meta description"
image: "https://example.com/og-image.jpg"
language: en
---
```

**Supported Property Types:**
- Text (`key: value`)
- List (`key: [a, b]`)
- Number (`key: 42`)
- Checkbox (`key: true`)
- Date (`key: YYYY-MM-DD`)
- Date & Time (`key: YYYY-MM-DDTHH:MM:SS`)
- Tags (`key: [#tag1, #tag2]`)

**Standard Fields for Web Clippings:**
`title`, `tags`, `aliases`, `created`, `source`/`url`, `author`, `description`, `image`, `published`, `domain`.

#### 1.1.2 Wikilinks (`[[Like This]]`)

Obsidian's native linking syntax:
- `[[Note Title]]`: Links to another note.
- `[[Note Title|Display Text]]`: Links with custom display text.
- `[[Note Title#Heading]]`: Links to a specific heading.
- `[[Note Title#^block-id]]`: Links to a specific block.
- `![[image.png]]`: Embeds an image or other file.

For scrapers, wikilinks are useful for cross-referencing clips from the same domain or linking to author notes.

#### 1.1.3 Callouts (Admonitions)

Special blockquote syntax for visually distinct content blocks:

```markdown
> [!note]
> This is a note callout.

> [!warning]
> This is a warning.
```

Foldable callouts are also supported:
```markdown
> [!note]- This is a foldable callout
> Content is hidden until expanded.
```

Callouts can be used to flag issues like WAF detection or content quality notes.

#### 1.1.4 Other Obsidian Flavored Markdown Extensions

Beyond standard CommonMark and GitHub Flavored Markdown (GFM), Obsidian supports:
- Embeds (`![[...]]`)
- Tags (`#tag-name`)
- MathJax/LaTeX (`$...$`, `$$...$$`)
- Footnotes (`[^1]`)
- Highlight (`==highlighted text==`)
- Strikethrough (`~~deleted~~`)
- Comments (`%% hidden comment %%`)
- Task lists (`- [ ]`, `- [x]`)

#### 1.1.5 Attachment Handling

Images and other assets are typically stored in a dedicated folder within the Obsidian vault (e.g., `_attachments/`, `assets/`). The Obsidian Web Clipper (v1.8.0+) supports saving images locally. Wikilink embed syntax (`![[filename.png]]`) is preferred for local assets.

#### 1.1.6 Folder Structure Conventions

Common Obsidian vault structures for web clippings include:
- `Vault/webclips/YYYY-MM-DD-title.md`
- `Vault/sources/domain.com/page-title.md`
- `Vault/webclips/_attachments/`

### 1.2 Competitor Matrix

Analysis of existing browser extensions and services reveals common features and approaches:

*   **Browser Extensions:** MarkDownload, Obsidian Web Clipper (Official), SingleFile. Key features include frontmatter support, local image handling, and templating.
*   **Read-it-Later Services:** Readwise Reader, Omnivore. Offer Obsidian export via integrations or plugins, with customizable frontmatter and highlight support.
*   **API-Based Scrapers:** Firecrawl, Jina Reader, Crawlee/Apify. Primarily focus on raw content extraction (Markdown, HTML, JSON) but often lack Obsidian-specific features like frontmatter.
*   **Notion → Obsidian Exporters:** Tools like `notion2obsidian` convert Notion content to Markdown, demonstrating successful conversion strategies.

#### 1.2.1 Key Insight: Defuddle

`Defuddle`, the content extraction engine used by the official Obsidian Web Clipper, is a significant benchmark. It produces clean Markdown with YAML frontmatter and handles various content types effectively. Its open-source nature and Rust implementation (via crates like `readability` and `html2md`) make it a strong candidate for adoption or inspiration.

## 2. User Research Report: Obsidian Web Scraping/Clipping

This report synthesizes user demand and pain points from Obsidian forums, GitHub issues, and Reddit communities to inform feature prioritization for Obsidian-compatible export.

### 2.1 Top User-Requested Features

Users consistently prioritize features that enhance workflow efficiency, content quality, and integration with Obsidian's ecosystem.

1.  **Duplicate Detection:** Warning or preventing the clipping of already-saved URLs.
2.  **PDF Clipping:** Extracting text and metadata from PDF documents.
3.  **Save Images Locally:** Downloading images to the vault and using local links.
4.  **Incremental Clipping:** Appending new highlights or content to existing notes.
5.  **Quick Save:** One-click saving to a default location without dialogs.
6.  **Mobile/iOS Workflow:** Streamlined clipping from mobile devices via share sheets and Shortcuts.
7.  **Template Logic:** Conditional statements and loops within templates.
8.  **Auto-Detect Content Type/Smart Templates:** Applying templates based on content or URL patterns.
9.  **Dataview-Compatible Properties:** Rich YAML frontmatter for querying.
10. **Background Processing:** Asynchronous clipping operations.
11. **Notification on Success/Failure:** User feedback on clipping status.
12. **Batch Clip All Open Tabs:** Saving multiple open tabs simultaneously.
13. **Code Block Preservation:** Accurate extraction of code snippets with syntax highlighting.
14. **LaTeX/Math Preservation:** Retaining mathematical formulas.
15. **Social Media/Platform-Specific Extraction:** Improved parsing for sites like X/Twitter, LinkedIn.

### 2.2 Pain Points Analysis

The most frequent user frustrations include:
- Duplicate clipping.
- Manual folder path selection.
- High friction in mobile clipping workflows.
- Broken image links due to lack of local saving.
- Failure to extract content from social media platforms.
- Loss of formatting for code blocks and mathematical content.

### 2.3 Feature Categories & Prioritization

Features are categorized into Workflow & UX, Content Quality, Obsidian Integration, AI/Smart Features, and Advanced Obsidian Features. A tiered approach (Tier 1: Quick Wins, Tier 2: Next Sprint) is recommended for phased implementation.

### 2.4 Unique Differentiators

The scraper's existing strengths offer unique advantages over competitors:
- **CLI-First Architecture:** Enables headless, server-side, and automated clipping workflows.
- **WAF/CAPTCHA Bypass:** Ability to handle protected sites, unlike most browser extensions.
- **Semantic Duplicate Detection:** Advanced detection using embeddings, beyond simple URL matching.
- **Streaming & Constant RAM Usage:** Efficiently handles large pages.
- **Sitemap-to-Vault Pipeline:** Batch processing of entire sites.
- **Auto-Generated MOCs:** Automatic creation of index pages for related content.
- **Git-Aware Sync:** Integration with Git-based vault workflows.
- **Content Type Auto-Detection:** Smarter template selection than basic URL regex.

## 3. Technical Design Suggestions for Obsidian Export

This section outlines a technical approach for implementing Obsidian-compatible Markdown export.

### 3.1 Architecture

The existing Clean Architecture pattern can be extended:

```mermaid
graph TD
    A[Domain Layer] --> B_Entities["B(Entities:"] OutputFormat::Markdown)
    A --> C_Exporter["C(Exporter"] Trait)
    A --> D_MarkdownDocument["D(MarkdownDocument"] Entity)
    E[Infrastructure Layer] --> F_Export["F(Export:"] MarkdownExporter)
    E --> G_HTML["G(HTML"] to Markdown Converter)
    H[Export Factory] --> I_Create_Exporter["I(Create_Exporter:"] Markdown Case)
```

### 3.2 Proposed `OutputFormat` Enum

A new enum to distinguish between RAG pipeline formats and file-per-URL formats:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    /// One JSONL file (for RAG pipelines)
    Jsonl,
    /// One markdown file per URL (Obsidian-compatible)
    Markdown,
}
```

### 3.3 `MarkdownExporter` Design

```rust
pub struct MarkdownExporter {
    config: MarkdownExporterConfig,
}

pub struct MarkdownExporterConfig {
    pub output_dir: PathBuf,
    pub attachments_dir: String,        // e.g., "_attachments"
    pub filename_strategy: FilenameStrategy,
    pub download_images: bool,
    pub use_wikilinks: bool,
}

pub enum FilenameStrategy {
    SlugifiedTitle,          // "article-title.md"
    DatePrefixed,            // "2026-04-03-article-title.md"
    DomainPath,              // "domain.com/path/to/page.md"
}
```

### 3.4 HTML → Markdown Conversion Strategy

**Recommended Stack:**
1.  **Content Extraction:** `readability` crate (Rust port of Mozilla Readability).
2.  **HTML → Markdown:** `html2md` or `mdka` crate.
3.  **Post-processing:** Fix links, handle images, add frontmatter.

### 3.5 Proposed Markdown Output Structure

```markdown
---
title: "Article Title"
source: "https://example.com/article"
created: 2026-04-03T17:00:00+00:00
author: "John Doe"
description: "Page excerpt or meta description"
tags:
  - web-clip
  - technology
domain: "example.com"
---

# Article Title

Main content converted to clean markdown...

## Section Heading

More content with proper formatting.

![Image description](_attachments/article-title-hero.png)

> [!warning] WAF Detected
> This page had Cloudflare protection. Content may be incomplete.

---
*Clipped from [example.com](https://example.com/article) on 2026-04-03*
```

### 3.6 Image Handling

Images will be downloaded asynchronously to a configurable `attachments_dir` (e.g., `_attachments/`) within the output directory. Markdown references will be updated to point to these local files using relative paths.

### 3.7 Integration with Existing Code

The `MarkdownExporter` will consume `ScrapedContent` (which already contains `title`, `content`, `url`, `excerpt`, `author`, `date`, `assets`). It will generate frontmatter, convert HTML to Markdown, download assets, and write the `.md` file. Integration with `StateStore` for incremental support is also planned.

## 4. Proposed GitHub Issue for Feature Implementation

A detailed GitHub issue outlines the problem, proposed solution, technical approach, and acceptance criteria for adding Obsidian-compatible Markdown export. This serves as a roadmap for development.

## 5. Appendix: Key Libraries & Tools Reference

A reference list of relevant libraries and tools used in the research and proposed implementation:

| Tool                  | Language   | Purpose                                     | License      |
| :-------------------- | :--------- | :------------------------------------------ | :----------- |
| **Defuddle**          | TypeScript | Content extraction → Markdown (Obsidian official) | MIT          |
| **Readability.js**    | JavaScript | Content extraction (Mozilla)                | Apache 2.0   |
| **Turndown**          | JavaScript | HTML → Markdown                             | MIT          |
| **readability**       | Rust       | Content extraction (Rust port)              | MIT          |
| **html2md**           | Rust       | HTML → Markdown                             | MIT          |
| **mdka**              | Rust       | HTML → Markdown (alternative)               | MIT          |
| **slug**              | Rust       | URL/filename slugification                  | MPL 2.0      |
| **serde_yaml**        | Rust       | YAML serialization                          | MIT/Apache   |
| **whatlang**          | Rust       | Language detection                          | MIT          |