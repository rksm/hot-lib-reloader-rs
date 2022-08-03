This examples shows to make hot reloading configurable using a feature.

For development use two terminals and run the binary

```shell
cargo watch -i lib -x 'run --features reload'
```

and (re-)build the lib

```shell
cargo watch -w lib -x build
```

To run with a statically compiled binary just do

```shell
cargo run
```
