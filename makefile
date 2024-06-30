WINDOWS_TARGET := x86_64-pc-windows-msvc
LINUX_TARGET := x86_64-unknown-linux-gnu
VERSION := $(shell sed -nE 's/^version\s?=\s?"(.*)"/\1/p' Cargo.toml)

.PHONY: none release win linux deb gh publish changelog fedora tty
none:
	@echo 'Explictly select "release" option'

release: linux
	@mkdir -p "release/$(VERSION)"
	cp "target/$(LINUX_TARGET)/release/nyaa" "release/$(VERSION)/nyaa-$(VERSION)-$(LINUX_TARGET)"
	@echo "\nCommits since last tag:"
	@git log $(shell git describe --tags --abbrev=0)..HEAD --oneline

win:
	cargo build --target $(WINDOWS_TARGET) --release

linux:
	cargo build --target $(LINUX_TARGET) --profile=github

fedora:
	@mkdir -p "release/$(VERSION)"
	cargo generate-rpm
	cp target/generate-rpm/nyaa-$(VERSION)*.rpm "release/$(VERSION)/"

deb:
	@mkdir -p "release/$(VERSION)"
	@echo Go grab a coffee...
	docker stop nyaa-deb || true
	docker rm nyaa-deb || true
	cd scripts; VERSION=$(VERSION) docker compose up
	cp "scripts/docker-deb/nyaa-$(VERSION)-x86_64.deb" "release/$(VERSION)/"

gh:
	gh release create v$(VERSION) release/$(VERSION)/* --draft --title v$(VERSION) --latest

changelog:
	@echo "Adds:"
	@git log $(shell git describe --tags --abbrev=0)..HEAD --oneline | sed -n 's/^.\+feat(\(.\+\))\?:\s\+/- /p'
	@echo
	@echo "Fixes:"
	@git log $(shell git describe --tags --abbrev=0)..HEAD --oneline | sed -n 's/^.\+fix(\(.\+\))\?:\s\+/- /p'

tty:
	vhs ./scripts/tty.tape

publish:
	@echo -n "Publish v$(VERSION) to crates.io? [y/N] " && read ans && [ $${ans:-N} = y ]
	cargo publish
