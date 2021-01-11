lint:
	cargo clippy

README.md: README.tpl src/lib.rs
	cargo readme > $@
	git add $@

test:
	# default features
	cargo test
	# all features
	cargo test --all-features

pre-release: test README.md

release-patch:
	cargo release patch

release-minor:
	cargo release minor

release-major:
	cargo release major

PHONY: lint test pre-release release-patch release-minor release-major
