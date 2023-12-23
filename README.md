# **[Kashiful Haque 🔗](https://ifkash.vercel.app)**

![Overview](assets/images/overview.png?raw=true)

## Docker 🐳
> Docker image available [here](https://hub.docker.com/r/notifkash/ifkash)

To run in a Docker container, follow the steps below 👇:

1. `docker build .` (Optionally, give it a tag)
2. `docker run -d -p <HOST_PORT>:80 <IMAGE_NAME>:<TAG>`

**Example** 🏁
```bash
docker build -t ifkash:1.0 .
docker run -d -p 3000:80 ifkash:1.0
```

To use Docker Compose service, follow the steps below 👇:
```bash
docker compose up --build -d
```
