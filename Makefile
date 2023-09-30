.PHONY: help run test

help:
	@echo make run: print advertisements
	@echo make test: run tests

run:
	cargo run -r

test:
	cargo test

install:
	cargo install --path .
