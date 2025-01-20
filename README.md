# memora

To start the server run
```bash
cargo run --bin memora
```

To automatically recompile on file change run
```bash
cargo watch -q -c -w src/ -x "run --bin memora"
```


To start an agent run
```bash
cargo run --bin agent -- --dir content --token <TOUR_TOKEN>
```

To automatically recompile on file change run
```bash
cargo watch -q -c -w src/ -x "run --bin agent -- --dir content --token <TOUR_TOKEN>"
```
