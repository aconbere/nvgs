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
-H "Nvgs-Username: test" \
-H "Nvgs-Password: pass" \
-X POST -d "{\"urls\": [\"http://anders.conbere.org\"]}" \
http://localhost:3000/crawls
```

## TODO
- Update crawls table to add who added the crawl
