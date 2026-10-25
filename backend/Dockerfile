FROM python:3.11-slim

# Install system dependencies required by MuJoCo headless & Pillow
RUN apt-get update && apt-get install -y --no-install-recommends \
    libgl1 \
    libglew2.2 \
    libosmesa6 \
    libglfw3 \
    && rm -rf /var/lib/apt/lists/*

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    MUJOCO_GL=osmesa \
    PORT=8080

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 8080

CMD ["sh", "-c", "uvicorn main:app --host 0.0.0.0 --port ${PORT:-8080}"]
