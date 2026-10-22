"""Development-center protocol tests; run: python -m unittest discover -s tools."""

import base64
import contextlib
import hashlib
import http.client
import json
import os
from pathlib import Path
import tempfile
import threading
import unittest

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

from demo_center import DemoCenter, MAX_REQUEST, SESSION_SECONDS


class CenterTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name) / "center"
        self.center = DemoCenter.init(self.root, "tester", "example-password", "example-passphrase")
        self.now = [2_000_000_000.0]
        self.center.clock = lambda: self.now[0]
        self.binary = Path(self.temporary.name) / "candidate"
        self.binary.write_bytes(b"test-executable-v1")
        self.evidence = {
            "candidate_id": "demo-1", "source_revision": "1" * 40,
            "artifact_sha256": hashlib.sha256(self.binary.read_bytes()).hexdigest(),
            "change_summary": "Exercise signed update", "risk": "low",
            "tests": [{"name": "health", "passed": True, "evidence_sha256": "2" * 64}],
            "approval": {"decision": "approved", "actor": "test-policy", "policy": "automatic-low-risk"},
            "rollback": "Restore the previous immutable slot",
            "migration": {"kind": "none", "reversible": True}, "protected_core_changed": False,
        }

    def tearDown(self):
        self.temporary.cleanup()

    def login(self, peer="test"):
        return self.center.login({"account": "tester", "password": "example-password", "passphrase": "example-passphrase"}, peer)

    @contextlib.contextmanager
    def running(self):
        server = self.center.http_server()
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            yield server.server_port
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)

    def request(self, port, method, path, body=None, token=None, headers=None):
        connection = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
        request_headers = dict(headers or {})
        if token:
            request_headers["Authorization"] = f"Bearer {token}"
        raw = json.dumps(body) if isinstance(body, dict) else body
        connection.request(method, path, body=raw, headers=request_headers)
        response = connection.getresponse()
        result = response.status, response.read()
        connection.close()
        return result

    def test_each_credential_required(self):
        base = {"account": "tester", "password": "example-password", "passphrase": "example-passphrase"}
        for index, field in enumerate(base):
            bad = dict(base)
            bad[field] = "wrong"
            status, result = self.center.login(bad, f"peer-{index}")
            self.assertEqual(status, 401)
            self.assertEqual(result, {"error": "authentication failed"})
        self.assertEqual(self.login()[0], 200)

    def test_credential_limits_match_rust_client(self):
        for account, password, passphrase in [("a" * 257, "p", "k"), ("line\nbreak", "p", "k"), ("a", "字" * 1366, "k")]:
            with self.assertRaises(ValueError):
                DemoCenter.init(Path(self.temporary.name) / "invalid", account, password, passphrase)
        escaped = DemoCenter.init(Path(self.temporary.name) / "escaped", "tester", "\0" * 4096, "\0" * 4096)
        server = escaped.http_server()
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            status, _ = self.request(server.server_port, "POST", "/v1/login", {"account": "tester", "password": "\0" * 4096, "passphrase": "\0" * 4096})
            self.assertEqual(status, 200)
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)

    def test_passwords_are_salted_hashes(self):
        stored = (self.root / "account.json").read_text()
        self.assertNotIn("example-password", stored)
        self.assertNotIn("example-passphrase", stored)
        values = json.loads(stored)
        self.assertNotEqual(values["password"]["salt"], values["passphrase"]["salt"])
        if os.name == "posix":
            self.assertEqual((self.root / "signing-key.bin").stat().st_mode & 0o777, 0o600)
            self.assertEqual(self.root.stat().st_mode & 0o777, 0o700)

    def test_sessions_expire_and_restart_revokes(self):
        status, result = self.login()
        token = result["access_token"]
        self.assertEqual(status, 200)
        self.assertTrue(self.center.authorized(f"Bearer {token}"))
        self.assertNotIn(token, self.center._sessions)
        self.assertFalse(DemoCenter(self.root).authorized(f"Bearer {token}"))
        self.now[0] += SESSION_SECONDS
        self.assertFalse(self.center.authorized(f"Bearer {token}"))

    def test_login_is_throttled(self):
        for _ in range(5):
            self.assertEqual(self.center.login({}, "peer")[0], 401)
        self.assertEqual(self.login("peer")[0], 429)
        self.now[0] += 61
        self.assertEqual(self.login("peer")[0], 200)

    def test_published_signature_binds_exact_artifact(self):
        envelope = self.center.publish(self.binary, 1, "0.1.1", "linux-x86_64", self.evidence)
        payload = base64.b64decode(envelope["payload"])
        public = Ed25519PublicKey.from_public_bytes(bytes.fromhex(self.center.public_key))
        public.verify(bytes.fromhex(envelope["signature"]), payload)
        with self.assertRaises(InvalidSignature):
            public.verify(bytes.fromhex(envelope["signature"]), payload + b" ")
        manifest = json.loads(payload)
        self.assertEqual(manifest["sha256"], self.evidence["artifact_sha256"])
        self.assertEqual(manifest["expires_at"], int(self.now[0]) + 3600)
        self.assertEqual(manifest["size"], self.binary.stat().st_size)
        with self.assertRaises(ValueError):
            self.center.publish(self.binary, 2, "0.1.2", "linux-x86_64", {**self.evidence, "artifact_sha256": "0" * 64})

    def test_published_sequence_increases_across_instances(self):
        self.center.publish(self.binary, 2, "0.1.2", "linux-x86_64", self.evidence)
        other = DemoCenter(self.root)
        for sequence in (1, 2):
            with self.assertRaises(ValueError):
                other.publish(self.binary, sequence, "0.1.1", "linux-x86_64", self.evidence)
        other.publish(self.binary, 3, "0.1.3", "linux-x86_64", self.evidence)

    def test_paths_require_auth_and_reject_traversal(self):
        self.center.publish(self.binary, 1, "0.1.1", "linux-x86_64", self.evidence)
        token = self.login()[1]["access_token"]
        with self.running() as port:
            for path in ("/v1/releases/latest?channel=stable&target=linux-x86_64", f"/artifacts/{self.evidence['artifact_sha256']}"):
                self.assertEqual(self.request(port, "GET", path)[0], 401)
                self.assertEqual(self.request(port, "GET", path, token=token)[0], 200)
            for path in ("/artifacts/../account.json", "/artifacts/%2e%2e/account.json", "/signing-key.bin"):
                self.assertEqual(self.request(port, "GET", path, token=token)[0], 404)
            self.assertEqual(self.request(port, "GET", "/v1/releases/latest?channel=../&target=linux-x86_64", token=token)[0], 400)
            self.assertEqual(self.request(port, "GET", "/v1/releases/latest?channel=stable&target=linux-x86_64&target=other", token=token)[0], 400)

    def test_corrupted_immutable_artifact_is_refused(self):
        self.center.publish(self.binary, 1, "0.1.1", "linux-x86_64", self.evidence)
        digest = self.evidence["artifact_sha256"]
        (self.root / "artifacts" / digest).write_bytes(b"corrupted")
        token = self.login()[1]["access_token"]
        with self.running() as port:
            self.assertEqual(self.request(port, "GET", f"/artifacts/{digest}", token=token)[0], 409)
        with self.assertRaises(ValueError):
            self.center.publish(self.binary, 2, "0.1.2", "linux-x86_64", self.evidence)

    def test_auth_body_limits_and_redacted_errors(self):
        with self.running() as port:
            status, body = self.request(port, "POST", "/v1/login", {"account": "tester", "password": "wrong-secret", "passphrase": "example-passphrase"})
            self.assertEqual(status, 401)
            self.assertNotIn(b"wrong-secret", body)
            self.assertNotIn(b"example-passphrase", body)
            self.assertEqual(self.request(port, "POST", "/v1/login", b"x" * (MAX_REQUEST + 1))[0], 413)
            self.assertEqual(self.request(port, "POST", "/v1/login", b"not-json")[0], 400)
            self.assertEqual(self.request(port, "POST", "/v1/login", b"{}", headers={"Transfer-Encoding": "chunked"})[0], 400)

    @unittest.skipUnless(os.name == "posix", "Unix private-file permission test")
    def test_insecure_private_files_rejected(self):
        os.chmod(self.root / "account.json", 0o644)
        with self.assertRaises(ValueError):
            DemoCenter(self.root)

    @unittest.skipUnless(os.name == "posix", "Symlink creation test")
    def test_symlink_artifact_rejected(self):
        self.center.publish(self.binary, 1, "0.1.1", "linux-x86_64", self.evidence)
        digest = self.evidence["artifact_sha256"]
        artifact = self.root / "artifacts" / digest
        artifact.unlink()
        artifact.symlink_to(self.binary)
        token = self.login()[1]["access_token"]
        with self.running() as port:
            self.assertEqual(self.request(port, "GET", f"/artifacts/{digest}", token=token)[0], 400)


if __name__ == "__main__":
    unittest.main()
