all: cleanlog archive ui

archive:
	cargo run -- archive

ui:
	cargo run -- ui

cleanlog:
	rm snippets.log
