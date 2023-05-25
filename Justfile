_default:
  just --list

run:
  podman run catscii:latest --env SENTRY_DSN --env SENTRY_DSN -p 8080:8080/tcp --rm catscii

deploy:
  buildah build -f Dockerfile -t catscii .
  podman push catscii docker://registry.fly.io/late-wood-6224:latest
  flyctl deploy -i registry.fly.io/late-wood-6224:latest
