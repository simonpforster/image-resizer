# Image Resizer
Stateless image resizing service.


## Building docker image
```
docker buildx build . \ 
    --file=./service/Dockerfile \
    --platform linux/amd64 \
    -t europe-west2-docker.pkg.dev/listen-and-learn-411214/image-resizer/image-resizer-service:v0.X
```