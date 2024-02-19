WINDOWS_TARGET := x86_64-pc-windows-msvc
LINUX_TARGET := x86_64-unknown-linux-gnu
VERSION := $(shell sed -nE 's/^version\s?=\s?"(.*)"/\1/p' Cargo.toml)

.PHONY: release none publish
none:
	@echo 'Explictly select "release" option'

release:
	@mkdir -p "release/$(VERSION)"
	cargo build --target $(WINDOWS_TARGET) --release
	cargo build --target $(LINUX_TARGET) --release
	cp "target/$(WINDOWS_TARGET)/release/nyaa.exe" "release/$(VERSION)/nyaa-$(VERSION)-$(WINDOWS_TARGET).exe"
	cp "target/$(LINUX_TARGET)/release/nyaa" "release/$(VERSION)/nyaa-$(VERSION)-$(LINUX_TARGET)"
	@echo "\nCommits since last tag:"
	@git log $(shell git describe --tags --abbrev=0)..HEAD --oneline

gh:
	gh release create v$(VERSION) release/$(VERSION)/* --draft

publish:
	python3 publish.py
