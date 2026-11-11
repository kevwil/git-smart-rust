.PHONY: help all clean update fmt lint build release test audit run

APP_NAME  = git-smart

## all           : run default tasks [fmt lint test build]
all: fmt lint test build

## clean         : remove build artifacts
clean:
	@echo "\n*** cargo clean\n"
	@cargo clean

## update        : update dependencies within Cargo.toml constraints
update: Cargo.toml
	@echo "\n*** update dependencies\n"
	@cargo update

## fmt           : format the code with rustfmt
fmt:
	@echo "\n*** rustfmt\n"
	@cargo fmt --all

## lint          : check formatting and run clippy (warnings are errors)
lint:
	@echo "\n*** lint the code\n"
	@cargo fmt --all -- --check
	@cargo clippy --all-targets --locked -- -D warnings

## test          : run unit tests
test:
	@echo "\n*** run tests\n"
	@cargo test --locked

## build         : dev build into target/release
build: Cargo.toml
	@echo "\n*** build app\n"
	@cargo build

## release       : optimized release build into target/release
release: Cargo.toml
	@echo "\n*** build app release\n"
	@cargo build --release --locked

## audit         : check dependencies for known vulnerabilities (needs: cargo install cargo-audit)
audit: Cargo.lock
	@echo "\n*** RustSec vulnerability check\n"
	@cargo audit --version >/dev/null 2>&1 || { echo "cargo-audit not found; install with: cargo install cargo-audit --locked"; exit 1; }
	@cargo audit

## run           : run the app using local code, e.g. make run ARGS=~/code
run: Cargo.toml
	@echo "\n*** run the app using local code (not a release binary)\n"
	@cargo run --locked -- $(ARGS)

## help          : print self-documented target info from this file
help: Makefile
	@sed -n 's/^##//p' $<
