all:
	asciidoctor -d book -D book/ index.adoc
	cp -r images book

clean:
	rm -rf book build

# --- PDF build (Docker) -----------------------------------------------------
# Two-stage pipeline:
#   1. asciidoctor (Docker)              : AsciiDoc -> DocBook XML (expands include::)
#   2. local pandoc/latex+fonts (Docker) : DocBook XML -> PDF via xelatex
#
# The PDF builder is a thin Dockerfile on top of pandoc/latex:latest that adds
# DejaVu fonts (Dockerfile.pandoc) — needed for Unicode glyphs used in code
# listings (box-drawing chars, check marks, etc.) that aren't in TeX's
# default Latin Modern.
#
# Usage:   make pdf
# Output:  build/embassy-book.pdf

PDF_OUT      ?= /tmp/embassy-book.pdf
PANDOC_IMG   ?= embassy-pandoc:latest
ASCIIDOC_IMG ?= asciidoctor/docker-asciidoctor:latest
DOCKER       ?= docker
UID          := $(shell id -u)
GID          := $(shell id -g)

# Fonts. Both families below are pre-installed in embassy-pandoc:latest:
#   DejaVu Serif / DejaVu Sans / DejaVu Sans Mono
#   IBM Plex Serif / IBM Plex Sans / IBM Plex Mono
# Override on the command line, e.g.:
#   make pdf MONO_FONT="IBM Plex Mono"
#   make pdf MAIN_FONT="IBM Plex Serif" SANS_FONT="IBM Plex Sans" MONO_FONT="IBM Plex Mono"
MAIN_FONT ?= DejaVu Serif
SANS_FONT ?= DejaVu Sans
MONO_FONT ?= DejaVu Sans Mono

ADOC_SOURCES := index.adoc $(wildcard pages/*.adoc)

# Mount the repo root (not just docs/) so the docs/examples/examples symlink
# (-> ../../examples) resolves to a path inside the bind mount.
REPO_ROOT := $(realpath $(CURDIR)/..)
DOCS_REL  := $(notdir $(CURDIR))

build/embassy-book.xml: $(ADOC_SOURCES)
	mkdir -p build
	$(DOCKER) run --rm -u $(UID):$(GID) \
		-v "$(REPO_ROOT):/workspace" -w "/workspace/$(DOCS_REL)" \
		$(ASCIIDOC_IMG) \
		asciidoctor -b docbook5 -d book -o build/embassy-book.xml index.adoc
	# Substitute a couple of Unicode chars that aren't in every monospace
	# font (e.g. IBM Plex Mono lacks U+2714 check / U+2717 cross), so the
	# output stays clean regardless of MONO_FONT.
	@perl -i -CSD -pe 's/\x{2714}/[x]/g; s/\x{2717}/[ ]/g' build/embassy-book.xml

build/.pandoc-image-id: Dockerfile.pandoc
	mkdir -p build
	$(DOCKER) build --platform=linux/amd64 -f Dockerfile.pandoc -t $(PANDOC_IMG) .
	$(DOCKER) image inspect --format '{{.Id}}' $(PANDOC_IMG) > $@

# pandoc/latex publishes amd64-only; force the platform so it runs under
# Rosetta on Apple Silicon.
# Build into the workspace mount (Docker Desktop on macOS exposes /tmp as
# root-owned and not writable by the host UID inside the container), then
# copy to PDF_OUT on the host.
pdf: build/embassy-book.xml build/.pandoc-image-id pandoc-header.tex
	@mkdir -p "$(dir $(PDF_OUT))"
	$(DOCKER) run --rm --platform=linux/amd64 -u $(UID):$(GID) \
		-e HOME=/tmp -e XDG_CACHE_HOME=/tmp/.cache \
		-v "$(REPO_ROOT):/workspace" -w "/workspace/$(DOCS_REL)" \
		$(PANDOC_IMG) \
		--from=docbook --to=latex \
		--pdf-engine=xelatex \
		--toc --toc-depth=2 \
		--number-sections \
		--syntax-highlighting=tango \
		--include-in-header=pandoc-header.tex \
		-V documentclass=book \
		-V geometry:margin=1in \
		-V colorlinks=true \
		-V linkcolor=NavyBlue \
		-V urlcolor=NavyBlue \
		-V toccolor=black \
		-V mainfont="$(MAIN_FONT)" \
		-V sansfont="$(SANS_FONT)" \
		-V monofont="$(MONO_FONT)" \
		-V title="The Embassy Book" \
		-V subtitle="A Rust framework for async embedded development" \
		-V author="The Embassy Project" \
		-V date="$(shell date +%Y-%m-%d)" \
		-o build/embassy-book.pdf build/embassy-book.xml
	cp build/embassy-book.pdf "$(PDF_OUT)"
	@echo "Wrote $(PDF_OUT)"

.PHONY: all clean pdf
