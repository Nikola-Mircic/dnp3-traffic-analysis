# OUTSTATION DOCKERFILE
FROM --platform=linux/amd64 python:3.10-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

ENV DNP3_PORT=20000
ENV PYTHONUNBUFFERED=1

EXPOSE 20000

CMD ["python", "app.py"]
