#!/usr/bin/env python3
"""Loopback-only GESA release/authentication center for protocol experiments.

State and credentials live under --root, never in command arguments or the repo.
The release signer is intentionally local to this development fixture; a deployed
center needs a separate offline/restricted signing service and HTTPS termination.
On Windows this fixture inherits directory ACLs: use a private per-user --root.
Its development signing key is a plaintext file, without DPAPI protection.
"""

from __future__ import annotations

import argparse
import base64
import contextlib
import getpass
import hashlib
import hmac
import json
import os
import re
import secrets
import stat
import sys
import tempfile
import threading
import time
from collections import defaultdict, deque
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any, Callable
from urllib.parse import parse_qs, urlsplit

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

MAX_ARTIFACT = 64 * 1024 * 1024
# Two 4096-byte credentials can expand up to sixfold under JSON escaping.
MAX_REQUEST = 64 * 1024
MAX_METADATA = 256 * 1024
SESSION_SECONDS = 15 * 60
NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
ARTIFACT_PATH = re.compile(r"/artifacts/([0-9a-f]{64})\Z")


def _json(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def _directory(path: Path) -> None:
    if path.is_symlink():
        raise ValueError("State directories must be regular directories")
    path.mkdir(mode=0o700, parents=True, exist_ok=True)
    if not path.is_dir():
        raise ValueError("State path must be a directory")
    if os.name == "posix":
        os.chmod(path, 0o700)


def _read(path: Path, limit: int, private: bool = False) -> bytes:
    """Read bounded regular files without following a leaf symlink."""
    if path.is_symlink():
        raise ValueError("Symlinks are not accepted")
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
    with os.fdopen(descriptor, "rb") as stream:
        info = os.fstat(stream.fileno())
        if not stat.S_ISREG(info.st_mode) or info.st_size > limit:
            raise ValueError("Invalid file type or size")
        if private and os.name == "posix" and info.st_mode & 0o077:
            raise ValueError("Private file permissions must be 0600")
        value = stream.read(limit + 1)
        if len(value) > limit:
            raise ValueError("File exceeds size limit")
        return value


def _write(path: Path, content: bytes) -> None:
    """Flush temporary bytes, atomically replace, then flush the directory."""
    if path.is_symlink():
        raise ValueError("Symlinks are not accepted")
    with tempfile.NamedTemporaryFile(prefix=".tmp-", dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        try:
            os.chmod(temporary, 0o600)
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    try:
        os.replace(temporary, path)
        if os.name == "posix":
            descriptor = os.open(path.parent, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            try:
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
    finally:
        temporary.unlink(missing_ok=True)


def _name(value: str) -> str:
    if not isinstance(value, str) or not NAME.fullmatch(value) or ".." in value:
        raise ValueError("Invalid channel or target")
    return value


@contextlib.contextmanager
def _publish_lock(root: Path):
    descriptor = os.open(root / ".publish.lock", os.O_CREAT | os.O_RDWR | getattr(os, "O_NOFOLLOW", 0), 0o600)
    with os.fdopen(descriptor, "r+b") as stream:
        if os.name == "nt":
            import msvcrt
            if os.fstat(stream.fileno()).st_size == 0:
                stream.write(b"0")
                stream.flush()
            stream.seek(0)
            msvcrt.locking(stream.fileno(), msvcrt.LK_LOCK, 1)
            try:
                yield
            finally:
                stream.seek(0)
                msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)
        else:
            import fcntl
            fcntl.flock(stream.fileno(), fcntl.LOCK_EX)
            try:
                yield
            finally:
                fcntl.flock(stream.fileno(), fcntl.LOCK_UN)


def _derive(value: str, salt: bytes) -> bytes:
    return hashlib.scrypt(value.encode("utf-8"), salt=salt, n=2**14, r=8, p=1, dklen=32)


def _credential(value: str) -> dict[str, str]:
    salt = secrets.token_bytes(16)
    return {"salt": salt.hex(), "hash": _derive(value, salt).hex()}


def _matches(value: str, stored: dict[str, str]) -> bool:
    return hmac.compare_digest(_derive(value, bytes.fromhex(stored["salt"])), bytes.fromhex(stored["hash"]))


def _valid_credentials(account: Any, password: Any, passphrase: Any) -> bool:
    if any(not isinstance(value, str) or not value or len(value.encode("utf-8")) > 4096 for value in (account, password, passphrase)):
        return False
    return bool(account.strip()) and len(account.encode("utf-8")) <= 256 and not any(ord(char) < 32 or 127 <= ord(char) <= 159 for char in account)


class DemoCenter:
    def __init__(self, root: Path | str, clock: Callable[[], float] = time.time):
        self.root = Path(root).absolute()
        if self.root.is_symlink() or not self.root.is_dir():
            raise ValueError("Initialize a regular center directory first")
        self.credentials = json.loads(_read(self.root / "account.json", MAX_REQUEST, private=True))
        self.clock = clock
        self._sessions: dict[str, float] = {}
        self._failures: dict[str, deque[float]] = defaultdict(deque)
        self._lock = threading.Lock()
        self.max_attempts = 5
        self.attempt_window = 60

    @classmethod
    def init(cls, root: Path | str, account: str, password: str, passphrase: str) -> "DemoCenter":
        if not _valid_credentials(account, password, passphrase):
            raise ValueError("Three non-empty credential fields are required")
        root = Path(root).absolute()
        _directory(root)
        with _publish_lock(root):
            if (root / "account.json").exists() or (root / "signing-key.bin").exists():
                raise ValueError("The center is already initialized")
            for child in ("artifacts", "releases"):
                _directory(root / child)
            key = Ed25519PrivateKey.generate()
            private = key.private_bytes(serialization.Encoding.Raw, serialization.PrivateFormat.Raw, serialization.NoEncryption())
            _write(root / "signing-key.bin", private)
            _write(root / "account.json", _json({
                "schema": 1,
                "account_hash": hashlib.sha256(account.encode()).hexdigest(),
                "password": _credential(password),
                "passphrase": _credential(passphrase),
            }))
        return cls(root)

    @property
    def public_key(self) -> str:
        return self._signing_key().public_key().public_bytes(serialization.Encoding.Raw, serialization.PublicFormat.Raw).hex()

    def _signing_key(self) -> Ed25519PrivateKey:
        return Ed25519PrivateKey.from_private_bytes(_read(self.root / "signing-key.bin", 32, private=True))

    def login(self, body: Any, peer: str) -> tuple[int, dict[str, Any]]:
        now = self.clock()
        with self._lock:
            attempts = self._failures[peer]
            while attempts and attempts[0] <= now - self.attempt_window:
                attempts.popleft()
            if len(attempts) >= self.max_attempts:
                return 429, {"error": "authentication temporarily unavailable"}
            # Count in-flight requests too, bounding concurrent expensive scrypt work.
            attempts.append(now)
        if not isinstance(body, dict) or set(body) != {"account", "password", "passphrase"}:
            return 401, {"error": "authentication failed"}
        if not _valid_credentials(body["account"], body["password"], body["passphrase"]):
            return 401, {"error": "authentication failed"}
        account_ok = hmac.compare_digest(hashlib.sha256(body["account"].encode()).hexdigest(), self.credentials["account_hash"])
        password_ok = _matches(body["password"], self.credentials["password"])
        passphrase_ok = _matches(body["passphrase"], self.credentials["passphrase"])
        if not (account_ok & password_ok & passphrase_ok):
            return 401, {"error": "authentication failed"}
        token = secrets.token_urlsafe(32)
        expires_at = int(now) + SESSION_SECONDS
        with self._lock:
            self._failures.pop(peer, None)
            self._sessions = {key: expiry for key, expiry in self._sessions.items() if expiry > now}
            if len(self._sessions) >= 1024:
                return 429, {"error": "authentication temporarily unavailable"}
            self._sessions[hashlib.sha256(token.encode()).hexdigest()] = expires_at
        return 200, {"access_token": token, "expires_at": expires_at}

    def authorized(self, authorization: str) -> bool:
        if not authorization.startswith("Bearer ") or len(authorization) > 256:
            return False
        token_hash = hashlib.sha256(authorization[7:].encode()).hexdigest()
        with self._lock:
            return self._sessions.get(token_hash, 0) > self.clock()

    def _release_path(self, channel: str, target: str) -> Path:
        _name(channel)
        _name(target)
        directory = self.root / "releases"
        if directory.is_symlink():
            raise ValueError("Symlinks are not accepted")
        return directory / f"{channel}--{target}.json"

    def publish(self, binary: Path | str, sequence: int, version: str, target: str,
                iteration: dict[str, Any], channel: str = "stable", contract_epoch: int = 1,
                supports_contracts: list[int] | None = None, expires_in: int = 3600) -> dict[str, str]:
        if type(sequence) is not int or sequence < 1 or type(contract_epoch) is not int or contract_epoch < 1:
            raise ValueError("Sequence and contract epoch must be positive integers")
        supports_contracts = [1] if supports_contracts is None else supports_contracts
        if not supports_contracts or any(type(epoch) is not int or epoch < 1 for epoch in supports_contracts):
            raise ValueError("Supported contracts must be positive integers")
        if contract_epoch not in supports_contracts or len(set(supports_contracts)) != len(supports_contracts):
            raise ValueError("Supported contracts must uniquely include the current contract")
        if not isinstance(iteration, dict) or not version or len(version) > 80 or not 1 <= expires_in <= 604800:
            raise ValueError("Invalid release metadata")
        content = _read(Path(binary), MAX_ARTIFACT)
        if not content:
            raise ValueError("An executable artifact must contain bytes")
        digest = hashlib.sha256(content).hexdigest()
        if iteration.get("artifact_sha256") != digest:
            raise ValueError("Iteration evidence must bind the exact artifact")
        release_path = self._release_path(channel, target)
        with self._lock, _publish_lock(self.root):
            if release_path.exists():
                old = json.loads(_read(release_path, MAX_METADATA))
                old_payload = json.loads(base64.b64decode(old["payload"], validate=True))
                if sequence <= old_payload["sequence"]:
                    raise ValueError("Published sequences must increase")
            payload = {
                "schema": 1, "sequence": sequence, "version": version, "channel": channel,
                "target": target, "artifact": f"/artifacts/{digest}", "sha256": digest,
                "size": len(content), "expires_at": int(self.clock()) + expires_in, "protocol": 1,
                "contract_epoch": contract_epoch, "supports_contracts": supports_contracts,
                "iteration": iteration,
            }
            encoded = _json(payload)
            if len(encoded) > MAX_METADATA // 2:
                raise ValueError("Release metadata exceeds size limit")
            envelope = {"payload": base64.b64encode(encoded).decode(), "signature": self._signing_key().sign(encoded).hex()}
            artifact_dir = self.root / "artifacts"
            if artifact_dir.is_symlink():
                raise ValueError("Symlinks are not accepted")
            artifact = artifact_dir / digest
            if artifact.exists():
                if not hmac.compare_digest(hashlib.sha256(_read(artifact, MAX_ARTIFACT)).hexdigest(), digest):
                    raise ValueError("Existing immutable artifact has changed")
            else:
                _write(artifact, content)
            _write(release_path, _json(envelope))
            return envelope

    def http_server(self, port: int = 0) -> ThreadingHTTPServer:
        center = self

        class Handler(BaseHTTPRequestHandler):
            server_version = "GESA-Demo/1"
            sys_version = ""

            def log_message(self, *_args: Any) -> None:
                # Request headers, bodies and tokens must never reach access logs.
                return

            def send_bytes(self, status: int, data: bytes, content_type: str) -> None:
                self.send_response(status)
                self.send_header("Content-Type", content_type)
                self.send_header("Content-Length", str(len(data)))
                self.send_header("Cache-Control", "no-store")
                self.send_header("X-Content-Type-Options", "nosniff")
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(data)
                self.close_connection = True

            def send_json(self, status: int, body: Any) -> None:
                self.send_bytes(status, _json(body), "application/json")

            def do_POST(self) -> None:
                if self.path != "/v1/login":
                    self.send_json(404, {"error": "not found"})
                    return
                if self.headers.get("Transfer-Encoding") or len(self.headers.get_all("Content-Length", [])) != 1:
                    self.send_json(400, {"error": "invalid request"})
                    return
                try:
                    size = int(self.headers["Content-Length"])
                except (TypeError, ValueError):
                    self.send_json(400, {"error": "invalid request"})
                    return
                if not 0 < size <= MAX_REQUEST:
                    self.send_json(413, {"error": "invalid request size"})
                    return
                try:
                    self.connection.settimeout(5)
                    raw = self.rfile.read(size)
                    if len(raw) != size:
                        raise ValueError("truncated body")
                    body = json.loads(raw)
                    status, response = center.login(body, self.client_address[0])
                except (ValueError, UnicodeError, TimeoutError):
                    self.send_json(400, {"error": "invalid request"})
                    return
                self.send_json(status, response)

            def do_GET(self) -> None:
                if not center.authorized(self.headers.get("Authorization", "")):
                    self.send_json(401, {"error": "authentication required"})
                    return
                parsed = urlsplit(self.path)
                try:
                    if parsed.path == "/v1/releases/latest":
                        query = parse_qs(parsed.query, keep_blank_values=True, strict_parsing=True)
                        if set(query) != {"channel", "target"} or any(len(value) != 1 for value in query.values()):
                            raise ValueError("invalid query")
                        path = center._release_path(query["channel"][0], query["target"][0])
                        self.send_bytes(200, _read(path, MAX_METADATA), "application/json")
                        return
                    match = ARTIFACT_PATH.fullmatch(parsed.path)
                    if match and not parsed.query:
                        if (center.root / "artifacts").is_symlink():
                            raise ValueError("symlink")
                        content = _read(center.root / "artifacts" / match[1], MAX_ARTIFACT)
                        if not hmac.compare_digest(hashlib.sha256(content).hexdigest(), match[1]):
                            self.send_json(409, {"error": "artifact integrity failure"})
                            return
                        self.send_bytes(200, content, "application/octet-stream")
                        return
                except FileNotFoundError:
                    self.send_json(404, {"error": "not found"})
                    return
                except (ValueError, OSError):
                    self.send_json(400, {"error": "invalid request"})
                    return
                self.send_json(404, {"error": "not found"})

        server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
        server.daemon_threads = True
        return server


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    initialize = commands.add_parser("init", help="Create credentials and a local signing key interactively")
    initialize.add_argument("--root", type=Path, required=True)
    serve = commands.add_parser("serve", help="Run a development-only loopback server")
    serve.add_argument("--root", type=Path, required=True)
    serve.add_argument("--port", type=int, default=8787)
    publish = commands.add_parser("publish", help="Sign and publish a tested executable")
    publish.add_argument("--root", type=Path, required=True)
    publish.add_argument("--binary", type=Path, required=True)
    publish.add_argument("--sequence", type=int, required=True)
    publish.add_argument("--version", required=True)
    publish.add_argument("--target", required=True)
    publish.add_argument("--iteration", type=Path, required=True)
    publish.add_argument("--channel", default="stable")
    publish.add_argument("--contract-epoch", type=int, default=1)
    publish.add_argument("--supports-contracts", type=int, nargs="+", default=[1])
    publish.add_argument("--expires-in", type=int, default=3600)
    args = parser.parse_args()
    try:
        if args.command == "init":
            center = DemoCenter.init(args.root, input("Account: ").strip(), getpass.getpass("Password: "), getpass.getpass("Passphrase: "))
            print(json.dumps({"initialized": True, "release_public_key": center.public_key}))
        elif args.command == "serve":
            server = DemoCenter(args.root).http_server(args.port)
            print(json.dumps({"center_url": f"http://127.0.0.1:{server.server_port}", "development_only": True}), flush=True)
            with contextlib.closing(server):
                try:
                    server.serve_forever()
                except KeyboardInterrupt:
                    pass
        elif args.command == "publish":
            iteration = json.loads(_read(args.iteration, MAX_METADATA // 2))
            envelope = DemoCenter(args.root).publish(args.binary, args.sequence, args.version, args.target, iteration,
                                                    args.channel, args.contract_epoch, args.supports_contracts, args.expires_in)
            payload = json.loads(base64.b64decode(envelope["payload"]))
            print(json.dumps({"published": True, "sequence": payload["sequence"], "sha256": payload["sha256"]}))
        return 0
    except (OSError, ValueError, KeyError, EOFError):
        print(json.dumps({"error": "center operation failed; check initialization, permissions and release inputs"}), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
