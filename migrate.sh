#!/bin/env bash
set -e
set -x

PROJDIR="$(cd "$(dirname "$0")"; pwd -P)"
docker run --rm --net=host -v "$PROJDIR/dbmigrate:/flyway/sql:ro" flyway/flyway:6 \
  -user=incrementor \
  -password=inc \
  -url=jdbc:postgresql://localhost:5432/incrementor \
  migrate
