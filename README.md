# Peer

> A simple P2P CLI demo application

## Build

```bash
cargo build --release
```

## Options

<pre>
-r, --period <SECONDS>   Sets the messaging period [default: 1]
-p, --port <PORT>     Sets the peer port
-c, --connect <HOST:PORT>  Sets the address of the remote peer
-h, --help      Print help
-V, --version   Print version
</pre>

## Usage

### Starting the first peer with messaging period 5 seconds at port 8080

```bash
./peer --period=5 --port=8080
```

### Starting the second peer which will connect to the first

```bash
./peer --period=6 --port=8081 --connect="127.0.0.1:8080"
```

### Starting the second peer which will connect to all the peers through the first

```bash
./peer --period=7 --port=8082 --connect="127.0.0.1:8080"
```
