# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Documentation build (The Steelbore Standard §8). The program itself is built
# with Cargo; this Makefile builds and installs the Texinfo manual.

PROJECT      = wiremix
TEXI         = doc/$(PROJECT).texi
INFO         = doc/$(PROJECT).info
HTML         = doc/$(PROJECT).html
PDF          = doc/$(PROJECT).pdf

MAKEINFO     ?= makeinfo
TEXI2PDF     ?= texi2pdf
INSTALL_INFO ?= install-info
INSTALL      ?= install

PREFIX       ?= /usr/local
INFODIR      ?= $(PREFIX)/share/info

.DEFAULT_GOAL := info
.PHONY: doc info html pdf install-doc clean

# Build all three output formats.
doc: info html pdf

info: $(INFO)
html: $(HTML)
pdf: $(PDF)

$(INFO): $(TEXI)
	$(MAKEINFO) --no-split -o $@ $<

$(HTML): $(TEXI)
	$(MAKEINFO) --html --no-split -o $@ $<

$(PDF): $(TEXI)
	$(TEXI2PDF) --quiet --build-dir=doc/.t2p -o $@ $<

# Install the Info manual and register it with the system Info directory.
install-doc: $(INFO)
	$(INSTALL) -d $(DESTDIR)$(INFODIR)
	$(INSTALL) -m 0644 $(INFO) $(DESTDIR)$(INFODIR)/$(PROJECT).info
	$(INSTALL_INFO) --info-dir=$(DESTDIR)$(INFODIR) \
		$(DESTDIR)$(INFODIR)/$(PROJECT).info

clean:
	rm -rf $(INFO) $(HTML) $(PDF) doc/.t2p
