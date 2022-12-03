## Simple Newsletter API

An API for creating a backend application in Rust using the actix
framework that handles the various functionalities of a newsletter.

Following along the Rust: Zero to Production book with some changes here
and there. 


## How to run

Save the following env variables in a .env file:
```shell
POSTGRES_USER=postgres
POSTGRES_PASSWORD=password
POSTGRES_DB=newsletter
```

- Build the image using the dockerfile
```shell
docker build --tag z2p --file Dockerfile .
```

- Compose with postgres using
```shell
docker-compose up 
```

- Run migrations using
```shell
DATABASE_URL=postgres://postgres:password@localhost:5432/newsletter sqlx migrate run
```

Make sure you use the correct database url based on your env variables.


## To test

Run:
```shell
curl --request POST --data 'name=le%20guin&email=ursula_le_guin%40hotmail.com' 127.0.0.1:8000/subscriptions -v
```

You should get a 200 response code.
