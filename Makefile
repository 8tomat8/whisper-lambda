run:
	cargo lambda watch -a 127.0.0.1 -p 9001 --manifest-path ./whisper/Cargo.toml

pull_models:
	./scripts/pull_models.sh
