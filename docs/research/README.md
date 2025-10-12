# FCDB Paper

This directory contains the LaTeX source for the paper on Enishi, a Functorial–Categorical Database (FCDB).

## Files

- `main.tex`: The main LaTeX file for the paper.
- `README.md`: This file.

## Building

To build the paper, you will need a LaTeX distribution (e.g., TeX Live, MiKTeX) with `pdflatex` and the following packages:
- `amsmath`
- `amssymb`
- `amsthm`
- `booktabs`
- `hyperref`
- `graphicx`
- `tikz`
- `tabularx`
- `enumitem`

You can compile the paper by running:
```bash
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```
