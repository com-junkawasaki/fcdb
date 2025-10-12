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

#### Checklist updates (paper sections to verify)
- Ensure `main.tex` contains:
  - Experimental Setup and Reproducibility
  - Anti-commutativity Measurement: Design and Results (with placeholders)
  - Complexity Assumptions and $O(1{+}\varepsilon)$ Read Bound
  - Security Model and Revocation Boundary
  - Preservation table extended with Formal $\to$ Invariant $\to$ Test mapping
- Appendices include Reproducibility Artifacts with script references to `validation/` and `loadtest/k6_3hop.js`.

#### Packaging notes
- If you add figures for measurement (heatmaps, ablation, throughput-latency curves), place them in this folder and reference via `\\includegraphics{...}` so `arxiv_pack.sh` auto-detects them.



---

## arXiv Metadata (copy/paste into submission form)

- **Title**: Functorial-Categorical Database: A Compositional Framework for Information Preservation and Anti-Commutativity Reduction
- **Authors**: Jun Kawasaki
- **Abstract**:
  Conventional database architectures often secure local consistency by discarding information, entangling correctness with loss. We introduce the Functorial-Categorical Database (FCDb), which models data operations as morphisms in a layered functor category and establishes a Complete Preserving Family (CPF) of projections spanning content invariance (CAS), capability, and ownership, with optional observational projections for local order (B+Tree), temporal history (append-only/LSM), and adjacency (Graph). We identify a minimal kernel (F_core = Own o Cap o CAS) that preserves information and collapses non-commutativity to the ethical grant/revoke boundary. Under adjoint lifts and a fibred structure, operational pairs commute in the categorical limit while ownership integrity and capability constraints are maintained. The framework connects to information geometry via projection interpretations and supports empirical validation without discarding semantic, temporal, or relational entropy.
- **Comments**: LaTeX; inline bibliography; no external graphics required (TikZ templates included); packaging script `arxiv_pack.sh` provided; manifest auto-generated.
- **Subjects**: cs.DB (Databases); cs.DS (Data Structures and Algorithms); cs.LO (Logic in Computer Science); cs.PL (Programming Languages)
- **ACM CCS (suggested)**: Information systems → Database theory; Theory of computation → Category theory; Software and its engineering → Functional languages
- **MSC (optional)**: 18C10 (Category theory in computer science), 68P15 (Database theory)
- **Keywords**: category theory, information geometry, ownership, capabilities, content addressing, commutativity, adjunctions, fibred categories, morphological databases

