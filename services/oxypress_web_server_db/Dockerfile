# oxypress_postgres_db/Dockerfile
FROM postgres:16-alpine

# Set custom environment variables or configurations if needed
# ENV POSTGRES_DB=oxypress_web_db

# Copy automated setup scripts into the postgres initialization hook directory
COPY ./init.sql /docker-entrypoint-initdb.d/
