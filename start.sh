#!/bin/bash

new_window() {
  alacritty -e bash -c "$1" &
}

#Database
systemctl start docker.service
new_window "docker-compose -p db-1 up"
read -p "In case DB is up please press ENTER to continue..." enter
new_window "docker exec -it db-1-db-1 psql -U tstuser -W tstdb"

#Server
new_window "git ls-files | entr -r cargo run"

new_window "firefox http://localhost:8080"

nvim
