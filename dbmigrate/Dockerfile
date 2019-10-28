FROM flyway/flyway

COPY *.sql /flyway/sql

ENTRYPOINT ["sh", "-c", "/flyway/flyway -user=$DB_USER -password=$DB_PASS -url=jdbc:postgresql://$DB_HOST:$DB_PORT/$DB_NAME -connectRetries=60 migrate"]
