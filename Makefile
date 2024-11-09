generate-protos:
	@echo "**Updating microsoft/durabletask-protobuf**"
	git submodule update --remote --merge --force
	@echo "**Compiling protos to src/genproto**"
	cargo run --bin proto-gen
	@echo "**Running fmt on generated protos**"
	rustfmt ./durabletask-proto/src/microsoft.durabletask.implementation.protobuf.rs
