import json
import asyncio
from typing import Optional
from urllib.parse import urlparse, parse_qs
from http.server import HTTPServer, BaseHTTPRequestHandler
from passport_gateway import PassportGateway

class APIGateway:
    def __init__(self, node_id: str, passport: Optional[PassportGateway] = None):
        self.node_id = node_id
        self.queries_processed = 0
        self.passport = passport

    async def start_http_server(self):
        server = HTTPServer(("0.0.0.0", 8080), lambda *args: self.RequestHandler(*args, gateway=self))
        # Keep server running or something in real app

    class RequestHandler(BaseHTTPRequestHandler):
        def __init__(self, *args, gateway=None, **kwargs):
            self.gateway = gateway
            super().__init__(*args, **kwargs)

        def do_GET(self):
            path = urlparse(self.path).path
            if path == "/v1/status":
                self.send_json(200, {"status": "ok"})
            elif path == "/v1/oracle/feeds":
                self.send_json(200, {"feeds": []})
            elif path.startswith("/v1/identity/passport"):
                params = parse_qs(urlparse(self.path).query)
                address = params.get("address", [None])[0]
                if not address:
                    self.send_error(400, "Missing address")
                    return
                proof = asyncio.run(self.gateway.passport.is_human(address))
                self.send_json(200, {
                    "address": proof.address,
                    "is_human": proof.is_human,
                    "score": proof.score,
                    "stamps": proof.stamps,
                    "orcid_verified": proof.orcid_verified,
                })
            elif path.startswith("/v1/dao/verify-voter"):
                params = parse_qs(urlparse(self.path).query)
                address = params.get("address", [None])[0]
                if not address:
                    self.send_error(400, "Missing address")
                    return
                can_vote = asyncio.run(self.gateway.passport.verify_dao_voter(address))
                self.send_json(200, {"address": address, "can_vote": can_vote})
            else:
                self.send_error(404, "Not found")

        def send_json(self, status: int, data: dict):
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(data).encode())
