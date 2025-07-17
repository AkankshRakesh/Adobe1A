# syntax=docker/dockerfile:1
# AMD64-compatible Python image for Adobe Hackathon
FROM --platform=linux/amd64 python:3.11-slim

# Install system dependencies for PyMuPDF
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        libmupdf-dev \
        libfreetype6-dev \
        libjpeg-dev \
        libopenjp2-7-dev \
        libgumbo-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Install Python deps
COPY requirements.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# Copy source
COPY . /app

# Default runtime
ENTRYPOINT ["python", "-u", "main.py"]
