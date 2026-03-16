all: archive ui

archive:
	cargo run -- archive

ui:
	cargo run -- ui
