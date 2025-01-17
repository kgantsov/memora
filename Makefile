client:
	cargo run --bin client
watch:
	cargo watch -q -c -w src/ -x "run --bin memora"
