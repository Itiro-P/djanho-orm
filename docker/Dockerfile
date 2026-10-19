FROM python:3.12-slim
WORKDIR /app
COPY app.py ./

RUN apt-get update \
    && apt-get install -y --no-install-recommends sqlite3 \
    && rm -rf /var/lib/apt/lists/*

CMD ["python", "app.py"]