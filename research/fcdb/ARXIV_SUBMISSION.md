### arXiv submission package: research/fcdb

This folder contains the LaTeX source for the Enishi FCDB paper and a helper script to prepare an arXiv-compliant tarball.

- Main file: `main.tex`
- Helper: `arxiv_pack.sh` (bundles sources, detected includes, graphics, and optional `.bib` files)

#### How to build locally

```bash
pdflatex main.tex
# if you add a .bib later, also run:
# bibtex main && pdflatex main.tex && pdflatex main.tex
```

#### Create the arXiv tarball

```bash
chmod +x ./arxiv_pack.sh
./arxiv_pack.sh
```

This creates a file like `arxiv_fcdb_YYYYMMDD.tar.gz` along with `arxiv_manifest.txt` listing included files.

Notes:
- Currently the paper uses an inline bibliography block, so no `.bib` is included.
- If you add `\bibliography{...}` or `\addbibresource{...}`, the packer will include all `*.bib` files in this directory by default.
- Graphics are autodetected from `\\includegraphics{...}` and common extensions: `pdf`, `png`, `jpg`, `jpeg`.
- EPS warning (arXiv): Convert `.eps` to `.pdf` yourself before compiling/submitting (e.g., `epstopdf fig.eps`). arXiv compiles with shell escape disabled, so automatic EPS→PDF conversion will not run.


