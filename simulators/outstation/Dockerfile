# OUTSTATION DOCKERFILE
FROM --platform=linux/amd64 python:3.10-slim

WORKDIR /app

COPY . .

RUN pip install --no-cache-dir -r requirements.txt

ENV DNP3_PORT=20000

EXPOSE 20000

CMD ["python", "app.py"]
