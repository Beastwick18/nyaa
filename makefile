WINDOWS_TARGET := x86_64-pc-windows-msvc
LINUX_TARGET := x86_64-unknown-linux-gnu
VERSION := $(shell sed -nE 's/^version\s?=\s?"(.*)"/\1/p' Cargo.toml)

.PHONY: none release win linux deb gh publish
none:
	@echo 'Explictly select "release" option'

release: linux
	@mkdir -p "release/$(VERSION)"
	# cp "target/$(WINDOWS_TARGET)/release/nyaa.exe" "release/$(VERSION)/nyaa-$(VERSION)-$(WINDOWS_TARGET).exe"
	cp "target/$(LINUX_TARGET)/release/nyaa" "release/$(VERSION)/nyaa-$(VERSION)-$(LINUX_TARGET)"
	@echo "\nCommits since last tag:"
	@git log $(shell git describe --tags --abbrev=0)..HEAD --oneline

win:
	cargo build --target $(WINDOWS_TARGET) --release

linux:
	cargo build --target $(LINUX_TARGET) --release

deb:
	@mkdir -p "release/$(VERSION)"
	@echo Go grab a coffee...
	docker stop nyaa-deb || true
	docker rm nyaa-deb || true
	VERSION=$(VERSION) docker compose up
	cp "docker-deb/nyaa-$(VERSION)-x86_64.deb" "release/$(VERSION)/"

gh:
	gh release create v$(VERSION) release/$(VERSION)/* --draft --title v$(VERSION) --latest

publish:
	python3 publish.py
