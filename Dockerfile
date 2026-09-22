FROM node:20-alpine

RUN apk add --no-cache git openssh-client docker-cli

WORKDIR /app/src

COPY package.json package-lock.json /app/
RUN npm ci --omit=dev --prefix /app && npm cache clean --force

COPY src/ /app/src/

EXPOSE 17380

ENTRYPOINT ["node", "/app/src/server.js"]
