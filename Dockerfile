FROM node:20-alpine

RUN apk add --no-cache git openssh-client docker-cli

WORKDIR /app/src

COPY src/ /app/src/

EXPOSE 17380

ENTRYPOINT ["node", "/app/src/server.js"]
