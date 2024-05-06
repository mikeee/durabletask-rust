generate-protos:
	@echo "**Updating microsoft/durabletask-protobuf**"
	git submodule update --remote --merge --force
	@echo "**Compiling protos to src/genproto**"
	cargo build --features genproto
	@echo "**Running fmt on generated protos**"
	rustfmt ./src/genproto/microsoft.durabletask.implementation.protobuf.rs
