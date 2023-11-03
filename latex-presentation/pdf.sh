#!/usr/bin/sh

DATE_COVER=$(date "+%d %B %Y")

SOURCE_FORMAT="markdown_strict\
+pipe_tables\
+backtick_code_blocks\
+auto_identifiers\
+tex_math_dollars\
+strikeout\
+yaml_metadata_block\
+implicit_figures\
+all_symbols_escapable\
+link_attributes\
+smart\
+fenced_divs"

PDF_ENGINE="lualatex"

pandoc -s --dpi=300 --slide-level 2 --toc --listings --shift-heading-level=0 \
    --template default_mod.latex --pdf-engine "${PDF_ENGINE}" -f "$SOURCE_FORMAT" \
    -M date="$DATE_COVER" -V classoption:aspectratio=169 -t beamer presentations-as-code.md -o presentations-as-code.pdf
