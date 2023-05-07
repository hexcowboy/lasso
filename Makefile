.PHONY: dev
dev:
	@RUST_LOG="lasso=trace" cargo watch -x run
