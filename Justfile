_default:
  just --list

deploy:
  DOCKER_BUILDKIT=1 docker build -t catscii .
  fly deploy --local-only
