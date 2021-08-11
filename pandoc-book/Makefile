FIGURES = $(shell find . -name '*.svg')

PANDOCFLAGS =                        \
	--table-of-contents                \
  --pdf-engine=tectonic                \
  --from=markdown                    \
  --number-sections                  \
  --indented-code-classes=javascript \
  --highlight-style=monochrome       \
  -V mainfont="Iosevka"             \
  -V documentclass=report            \
  -V papersize=A5                    \
  -V geometry:margin=1in

.PHONY: clean

output/%.pdf: %.md $(FIGURES) Makefile | output
        pandoc $< -o $@ $(PANDOCFLAGS)

