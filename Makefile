lint:
	cargo clippy

test:
	# default features
	cargo test
	# all features
	cargo test --all-features

release-patch:
	cargo release patch

release-minor:
	cargo release minor

release-major:
	cargo release major

PHONY: lint, test, release-patch, release-minor, release-major
