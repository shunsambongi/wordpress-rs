@_default:
    just --list

compose *args:
    docker compose -f docker/docker-compose.yml -p wordprs {{args}}
