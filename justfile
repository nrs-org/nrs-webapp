dev:
  bacon dev

wasm:
  bacon build-core-wasm

tailwind:
  pnpm --prefix nrs-webapp-frontend tailwind

postgres:
  docker run --rm --name pg -p 3622:5432 -e POSTGRES_PASSWORD=password -d postgres:17

psql:
  docker exec -it -u postgres pg psql

keygen:
  cargo run -p nrs-webapp-keygen
