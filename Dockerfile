FROM node:20-alpine

ARG SKRYNIA_VERSION=development
ARG SKRYNIA_COMMIT=development

RUN apk add --no-cache git openssh-client docker-cli

WORKDIR /app/src

COPY package.json package-lock.json /app/
RUN npm ci --omit=dev --prefix /app && npm cache clean --force

COPY src/ /app/src/

RUN printf '%s\n' "$SKRYNIA_VERSION" > /app/VERSION \
    && printf '%s\n' "$SKRYNIA_COMMIT" > /app/COMMIT

EXPOSE 17380

ENTRYPOINT ["node", "/app/src/server.js"]
