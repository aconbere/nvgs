## Getting Started

Initialize the index

```bash
./target/release/cli -path ~/path/to/index init
```

Start the webserver

```bash
./target/release/api --path ~/path/to/index --address 127.0.0.1:3456
```

## Test Queries
```bash
./target/release/cli -path ~/path/to/index init
```

```bash
./target/release/cli -path ~/path/to/index add-user --username test --password pass
```

```bash
curl -i \
-H "Content-Type: application/json" \
-H "Nvgs-Username: username" \
-H "Nvgs-Password: password" \
-X POST -d "{\"urls\": [\"http://anders.conbere.org\"]}" \
http://localhost:3000/crawls
```

## Deploying

Deploy tooling and descriptions are currently found in https://github.com/aconbere/nvgs-service (note this is currently a private repository, reach out to me if you need access).


