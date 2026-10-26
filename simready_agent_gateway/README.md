# SimReady Managed Agent API Gateway

## Setup

1. Install requirements:
```bash
pip install fastapi uvicorn pydantic
```

2. Run the gateway:
```bash
uvicorn gateway:app --reload
```

3. Test the endpoint (Requires X-API-Key header):
```bash
curl -X POST "http://localhost:8000/api/v1/agents/simready_fetcher/execute" \
     -H "X-API-Key: token_admin" \
     -H "Content-Type: application/json" \
     -d '{"asset_id": "apple_crate_01"}'
```
