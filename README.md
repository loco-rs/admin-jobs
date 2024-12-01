# Loco Jobs Admin
![title](media/screenshot.png)

Loco Jobs Admin provides simple and efficient background processing for the [Loco](http://github.com/loco-rs/loco) framework.
It supports background queues for both Postgres and SQLite databases.

# Setup

## Build the Docker Image
### 1. After cloning the project, you can build the Docker image using the following command:
```sh
docker build -t loco-job-admin .
```

### 2. Once the image is built, run the container and access the admin interface at `http://127.0.0.1:8080/`:
```sh
docker run -it -p 8080:5150 -e LOCO_ENV=development loco-job-admin
```

### 3. Configure the Database Connection
To configure the database connection, set the appropriate Docker environment variables:

* `WORKER_KIND`: Choose between Postgres or Sqlite.
* `CONNECTION_STRING`: Provide the worker's database connection string.


Example:
```sh
docker run -it -p 8080:5150 -e WORKER_KIND=Postgres -e CONNECTION_STRING="postgres://user:password@localhost/dbname" loco-job-admin
```