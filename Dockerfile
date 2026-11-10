FROM node:20-alpine
ARG SKRYNIA_VERSION=development
ARG SKRYNIA_COMMIT=development
WORKDIR /app/src
COPY package.json package-lock.json /app/
RUN npm ci --omit=dev --prefix /app && npm cache clean --force
COPY src/ /app/src/
COPY scripts/root-credentials.js /app/scripts/root-credentials.js
RUN printf '%s\n' "$SKRYNIA_VERSION" > /app/VERSION \
    && printf '%s\n' "$SKRYNIA_COMMIT" > /app/COMMIT
ENV SKRYNIA_HOST=0.0.0.0
EXPOSE 17380
ENTRYPOINT ["node", "/app/src/server.js"]
