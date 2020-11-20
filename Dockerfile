FROM python:3.8-alpine
RUN adduser --disabled-password --gecos "" faqbot && mkdir /src && chown faqbot:faqbot /src
RUN apk update && apk add gcc musl-dev libffi-dev openssl-dev && pip install poetry
USER faqbot
WORKDIR /src
COPY --chown=faqbot pyproject.toml poetry.lock /src/
RUN poetry install --no-dev
COPY --chown=faqbot . /src/
ENTRYPOINT poetry run python bot.py
