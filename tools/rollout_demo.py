#!/usr/bin/env python3
"""Local Redis-coordinated blue/green laboratory; not a production deployer.

Use an external JSON configuration: {"redis_host":"127.0.0.1",
"redis_port":6379,"namespace":"gesa-demo","requests":100}.
Optional redis_username/redis_password also stay in that external file.
Only numeric loopback Redis hosts are accepted. The demo owns and deletes only
its unique per-run namespace. It uses real HTTP backends and a stable gateway.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import ipaddress
import json
import os
from pathlib import Path
import secrets
import socket
import stat
import threading
import time
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class RedisError(RuntimeError):
    pass


class Redis:
    """Minimal bounded RESP2 client. One connection per command avoids sharing."""

    def __init__(self, host="127.0.0.1", port=6379, username=None, password=None):
        if not ipaddress.ip_address(host).is_loopback:
            raise ValueError("The laboratory requires numeric loopback Redis")
        self.host, self.port = host, int(port)
        self.username, self.password = username, password

    @staticmethod
    def _encode(args):
        pieces = [f"*{len(args)}\r\n".encode()]
        for arg in args:
            raw = str(arg).encode()
            pieces.extend([f"${len(raw)}\r\n".encode(), raw, b"\r\n"])
        return b"".join(pieces)

    @classmethod
    def _read(cls, stream, depth=0):
        if depth > 8:
            raise RedisError("Redis nesting limit")
        line = stream.readline(8193)
        if len(line) > 8192 or not line.endswith(b"\r\n"):
            raise RedisError("Invalid Redis response")
        kind, body = line[:1], line[1:-2]
        if kind == b"-":
            raise RedisError("Redis rejected a command")
        if kind == b"+":
            return body.decode()
        if kind == b":":
            return int(body)
        if kind == b"$":
            size = int(body)
            if size == -1:
                return None
            if not 0 <= size <= 1_000_000:
                raise RedisError("Redis bulk limit")
            raw = stream.read(size + 2)
            if len(raw) != size + 2 or not raw.endswith(b"\r\n"):
                raise RedisError("Truncated Redis response")
            return raw[:-2].decode()
        if kind == b"*":
            size = int(body)
            if not 0 <= size <= 1024:
                raise RedisError("Redis array limit")
            return [cls._read(stream, depth + 1) for _ in range(size)]
        raise RedisError("Unknown Redis response")

    def command(self, *args):
        with socket.create_connection((self.host, self.port), timeout=3) as conn:
            conn.settimeout(3)
            stream = conn.makefile("rb")
            try:
                if self.password is not None:
                    auth = ("AUTH", self.username, self.password) if self.username else ("AUTH", self.password)
                    conn.sendall(self._encode(auth))
                    self._read(stream)
                conn.sendall(self._encode(args))
                return self._read(stream)
            finally:
                stream.close()


SWITCH = """
if redis.call('GET', KEYS[1]) ~= ARGV[1] then return {0, 'lease'} end
local current = redis.call('GET', KEYS[2]) or ''
if current ~= ARGV[2] then return {0, 'stale'} end
local candidate = cjson.decode(ARGV[3])
if redis.call('HGET', KEYS[4], candidate.id) == 'retired' then
  return {0, 'retired'}
end
candidate.generation = redis.call('INCR', KEYS[3])
local encoded = cjson.encode(candidate)
redis.call('SET', KEYS[2], encoded)
return {1, encoded}
"""
UNLOCK = """
if redis.call('GET', KEYS[1]) == ARGV[1] then
  return redis.call('DEL', KEYS[1])
end
return 0
"""
# A request reserves its selected route in the SAME Redis operation as reading
# it. After switching, no new request can reserve the retired instance.
RESERVE = """
local raw = redis.call('GET', KEYS[1])
if not raw then return false end
local route = cjson.decode(raw)
redis.call('HINCRBY', KEYS[2], route.id, 1)
return raw
"""
RELEASE = """
local n = tonumber(redis.call('HGET', KEYS[1], ARGV[1]) or '0')
if n > 0 then return redis.call('HINCRBY', KEYS[1], ARGV[1], -1) end
return 0
"""
RETIRE = """
local raw = redis.call('GET', KEYS[1])
if raw and cjson.decode(raw).id == ARGV[1] then return 0 end
local n = tonumber(redis.call('HGET', KEYS[2], ARGV[1]) or '0')
if n ~= 0 then return 0 end
redis.call('HSET', KEYS[3], ARGV[1], 'retired')
return 1
"""


def response(handler, status, value):
    body = json.dumps(value, separators=(",", ":")).encode()
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


class HTTPServer(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True


class Backend:
    def __init__(self, version, delay=0.03, healthy=True, fail_after_checks=None):
        self.id = secrets.token_hex(12)
        self.version, self.delay, self.healthy = version, delay, healthy
        self.fail_after_checks = fail_after_checks
        self.checks = 0
        self.stopped = False
        backend = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_GET(self):
                if self.path == "/health":
                    backend.checks += 1
                    valid = backend.healthy and (
                        backend.fail_after_checks is None
                        or backend.checks <= backend.fail_after_checks
                    )
                    response(self, 200 if valid else 503, {"healthy": valid})
                    return
                time.sleep(backend.delay)
                response(self, 200, {"version": backend.version, "instance": backend.id})

        self.server = HTTPServer(("127.0.0.1", 0), Handler)
        self.port = self.server.server_address[1]
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def route(self):
        return {"id": self.id, "endpoint": f"http://127.0.0.1:{self.port}", "version": self.version}

    def ready(self):
        try:
            with urllib.request.urlopen(self.route()["endpoint"] + "/health", timeout=2) as result:
                return result.status == 200 and json.load(result).get("healthy") is True
        except (OSError, ValueError, urllib.error.URLError):
            return False

    def stop(self):
        if not self.stopped:
            self.server.shutdown()
            self.server.server_close()
            self.thread.join(2)
            self.stopped = True


class Rollout:
    def __init__(self, redis, namespace):
        if not namespace or len(namespace) > 100 or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_" for c in namespace):
            raise ValueError("Use a short alphanumeric Redis namespace")
        self.redis = redis
        # Redis hash tag also keeps all keys in one cluster slot.
        prefix = "gesa:{" + namespace + "}:"
        self.lease, self.route_key = prefix + "lease", prefix + "route"
        self.generation, self.inflight = prefix + "generation", prefix + "inflight"
        self.lifecycle = prefix + "lifecycle"
        self.backends = {}
        self.previous = None
        self.events = []
        rollout = self

        class Gateway(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass

            def do_GET(self):
                selected = None
                try:
                    raw = rollout.redis.command("EVAL", RESERVE, 2, rollout.route_key, rollout.inflight)
                    if raw is None:
                        response(self, 503, {"error": "No active route"})
                        return
                    selected = json.loads(raw)
                    # Endpoints come only from local Backend objects in this lab.
                    with urllib.request.urlopen(selected["endpoint"] + "/work", timeout=5) as upstream:
                        result = json.load(upstream)
                    response(self, 200, result)
                except (OSError, ValueError, RedisError, urllib.error.URLError):
                    response(self, 503, {"error": "Upstream unavailable"})
                finally:
                    if selected:
                        try:
                            rollout.redis.command("EVAL", RELEASE, 1, rollout.inflight, selected["id"])
                        except (OSError, RedisError):
                            # Lost accounting conservatively prevents retirement.
                            pass

        self.gateway = HTTPServer(("127.0.0.1", 0), Gateway)
        self.url = f"http://127.0.0.1:{self.gateway.server_address[1]}"
        self.gateway_thread = threading.Thread(target=self.gateway.serve_forever, daemon=True)
        self.gateway_thread.start()

    def current_raw(self):
        return self.redis.command("GET", self.route_key) or ""

    def current(self):
        return json.loads(self.current_raw() or "null")

    def compare_switch(self, expected, candidate):
        owner = secrets.token_hex(24)
        if self.redis.command("SET", self.lease, owner, "NX", "PX", 5000) != "OK":
            raise RedisError("Another updater holds the coordination lease")
        try:
            result = self.redis.command("EVAL", SWITCH, 4, self.lease, self.route_key, self.generation, self.lifecycle,
                                        owner, expected, json.dumps(candidate.route(), separators=(",", ":")))
            if result[0] != 1:
                raise RedisError("Coordination lease expired or route changed")
            self.events.append({"phase": "switched", "route": json.loads(result[1])})
            return result[1]
        finally:
            self.redis.command("EVAL", UNLOCK, 1, self.lease, owner)

    def outstanding(self, instance_id):
        return int(self.redis.command("HGET", self.inflight, instance_id) or 0)

    def drain(self, backend, timeout=3):
        deadline = time.monotonic() + timeout
        while True:
            current = self.current()
            if current and current["id"] == backend.id:
                return False
            # Retirement and activation share a Redis lifecycle fence. A
            # competing rollback cannot reactivate a process while it stops.
            retired = self.redis.command("EVAL", RETIRE, 3, self.route_key, self.inflight, self.lifecycle, backend.id)
            if retired == 1:
                backend.stop()
                self.events.append({"phase": "drained", "instance": backend.id})
                return True
            if time.monotonic() >= deadline:
                self.events.append({"phase": "retained_busy", "instance": backend.id})
                return False
            time.sleep(0.01)

    def install(self, candidate, drain_timeout=3):
        expected = self.current_raw()
        self.backends[candidate.id] = candidate
        if not candidate.ready():
            candidate.stop()
            self.events.append({"phase": "rejected_unhealthy", "instance": candidate.id})
            return {"committed": False, "rolled_back": False}
        try:
            activated = self.compare_switch(expected, candidate)
        except Exception:
            candidate.stop()
            raise
        old_route = json.loads(expected) if expected else None
        old = self.backends.get(old_route["id"]) if old_route else None
        if not candidate.ready():
            if old is None:
                # First install has no last-known-good route. Keep the process
                # for inspection; caller can close the isolated laboratory.
                raise RuntimeError("Initial backend failed its post-switch check")
            self.compare_switch(activated, old)
            self.drain(candidate, drain_timeout)
            self.events.append({"phase": "rolled_back", "instance": candidate.id})
            return {"committed": False, "rolled_back": True}
        self.previous = old
        drained = self.drain(old, drain_timeout) if old else True
        return {"committed": True, "rolled_back": False, "old_drained": drained}

    def close(self):
        self.gateway.shutdown()
        self.gateway.server_close()
        self.gateway_thread.join(2)
        for backend in self.backends.values():
            backend.stop()
        self.redis.command("DEL", self.lease, self.route_key, self.generation, self.inflight, self.lifecycle)


def load_config(path):
    if path.is_symlink():
        raise ValueError("Configuration must be a regular owned file")
    info = path.stat()
    if not stat.S_ISREG(info.st_mode):
        raise ValueError("Configuration must be a regular file")
    config = json.loads(path.read_text())
    if config.get("redis_password") and os.name == "posix" and info.st_mode & 0o077:
        raise ValueError("Configuration containing credentials must have mode 0600")
    return config


def exercise(config):
    redis = Redis(config.get("redis_host", "127.0.0.1"), config.get("redis_port", 6379),
                  config.get("redis_username"), config.get("redis_password"))
    redis.command("PING")
    namespace = config.get("namespace", "gesa-demo") + "-" + secrets.token_hex(6)
    lab = Rollout(redis, namespace)
    count = int(config.get("requests", 100))
    if not 10 <= count <= 10000:
        raise ValueError("requests must be between 10 and 10000")
    results, failures = [], []
    try:
        first = Backend("1.0.0")
        lab.install(first)

        def request_one(_):
            try:
                with urllib.request.urlopen(lab.url + "/work", timeout=5) as result:
                    return json.load(result)
            except Exception as error:
                return {"error": type(error).__name__}

        with concurrent.futures.ThreadPoolExecutor(max_workers=12) as executor:
            jobs = [executor.submit(request_one, index) for index in range(count)]
            deadline = time.monotonic() + 3
            while lab.outstanding(first.id) == 0 and time.monotonic() < deadline:
                time.sleep(0.005)
            switched = lab.install(Backend("2.0.0"))
            for job in jobs:
                item = job.result()
                (failures if "error" in item else results).append(item)
        rejected = lab.install(Backend("3.0.0-unhealthy", healthy=False))
        rollback = lab.install(Backend("3.0.0-fails-post-check", fail_after_checks=1))
        return {"ok": not failures and switched["committed"] and rollback["rolled_back"],
                "requests": count, "completed": len(results), "failures": len(failures),
                "versions_observed": sorted({r["version"] for r in results}),
                "active_version": lab.current()["version"], "unhealthy_rejected": not rejected["committed"],
                "post_activation_rollback": rollback["rolled_back"], "events": lab.events,
                "scope": "localhost HTTP blue-green laboratory with real Redis; no production module deployment"}
    finally:
        lab.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = exercise(load_config(args.config))
        print(json.dumps(result, indent=2))
        return 0 if result["ok"] else 1
    except Exception as error:
        # Exceptions may contain connection configuration; report only a class.
        print(json.dumps({"ok": False, "error": type(error).__name__}))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
