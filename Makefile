all: cleanlog archive ui

archive:
	cargo run -- archive ../data/snippets

ui:
	cargo run -- ui

cleanlog:
	rm -f snippets.log

release:
	cargo build --release

produi: release
	./target/release/snippets.exe ui