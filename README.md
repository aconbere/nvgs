## Test Queries

```bash
curl -i \
-H "Content-Type: application/json" \
-H "NVGS-USERNAME: test" \
-H "NVGS-PASSWORD: pass" \
-X POST -d "{\"urls\": [\"http://anders.conbere.org\"]}" \
http://localhost:3000/add
```

## TODO

- Add cli to add users to the database
- figure out hashing passwords
- Finish auth backend
- deploy cli and api binaries
