# Demo PDFs

Beamer decks used to check that pdfpc matches a normal PDF viewer (e.g. Poppler/`pdftoppm` or Evince).

| File | Purpose |
|---|---|
| `pdfpc-demo.pdf` | Tiny 3-page smoke deck |
| `beamer-links.pdf` | Clickable TOC for internal PDF links / history |
| `beamer-default-white.pdf` | Explicit white canvas, math, lists |
| `beamer-madrid.pdf` | Madrid theme (colored header/footer) |
| `beamer-dark.pdf` | Black canvas / light text |
| `beamer-boxes.pdf` | Warsaw blocks, table, TikZ |

Rebuild:

```bash
cd demo
for f in pdfpc-demo beamer-links beamer-default-white beamer-madrid beamer-dark beamer-boxes; do
  pdflatex -interaction=nonstopmode "$f.tex"
done
```

Reference render (system viewer / Poppler):

```bash
pdftoppm -png -r 100 beamer-madrid.pdf /tmp/madrid
```

Compare with:

```bash
cargo run --manifest-path ../rust/Cargo.toml -- --windowed beamer-madrid.pdf
```

Datasheets that omit an opaque page fill (many Distiller exports) must still appear on white paper in pdfpc — that is covered by the white “paper” layer behind `PdfPageImage`.
