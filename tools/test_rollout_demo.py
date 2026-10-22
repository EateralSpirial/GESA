"""Integration tests require a real redis-server (PATH or REDIS_SERVER)."""
import concurrent.futures
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import time
import unittest
import urllib.request

from rollout_demo import Backend, Redis, RedisError, Rollout, SWITCH, exercise


SERVER = os.environ.get("REDIS_SERVER") or shutil.which("redis-server")


@unittest.skipUnless(SERVER, "real Redis binary unavailable; set REDIS_SERVER")
class RedisRolloutTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory(prefix="gesa-rollout-test-")
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            cls.port = sock.getsockname()[1]
        cls.proc = subprocess.Popen([SERVER, "--bind", "127.0.0.1", "--port", str(cls.port),
                                     "--save", "", "--appendonly", "no", "--dir", cls.tmp.name],
                                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        cls.redis = Redis(port=cls.port)
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            try:
                if cls.redis.command("PING") == "PONG":
                    return
            except OSError:
                time.sleep(0.02)
        cls.proc.terminate()
        cls.proc.wait(5)
        raise RuntimeError("real Redis failed to start")

    @classmethod
    def tearDownClass(cls):
        cls.proc.terminate()
        cls.proc.wait(5)
        cls.tmp.cleanup()

    def setUp(self):
        self.lab = Rollout(self.redis, "test-" + self.id().rsplit(".", 1)[-1])

    def tearDown(self):
        self.lab.close()

    def get(self):
        with urllib.request.urlopen(self.lab.url + "/work", timeout=5) as result:
            return json.load(result)

    def test_concurrent_requests_survive_update(self):
        old = Backend("v1", delay=0.15)
        self.lab.install(old)
        with concurrent.futures.ThreadPoolExecutor(max_workers=16) as pool:
            running = [pool.submit(self.get) for _ in range(16)]
            deadline = time.monotonic() + 3
            while self.lab.outstanding(old.id) < 3 and time.monotonic() < deadline:
                time.sleep(0.005)
            self.assertGreaterEqual(self.lab.outstanding(old.id), 3)
            result = self.lab.install(Backend("v2"))
            running.extend(pool.submit(self.get) for _ in range(32))
            responses = [job.result() for job in running]
        self.assertTrue(result["committed"])
        self.assertTrue(old.stopped)
        self.assertEqual(len(responses), 48)
        self.assertEqual({item["version"] for item in responses}, {"v1", "v2"})
        self.assertEqual(self.lab.current()["version"], "v2")

    def test_unhealthy_candidate_preserves_old_route(self):
        old = Backend("v1")
        self.lab.install(old)
        before = self.lab.current_raw()
        bad = Backend("bad", healthy=False)
        result = self.lab.install(bad)
        self.assertFalse(result["committed"])
        self.assertEqual(before, self.lab.current_raw())
        self.assertTrue(bad.stopped)
        self.assertFalse(old.stopped)
        self.assertEqual(self.get()["version"], "v1")

    def test_post_switch_failure_rolls_back(self):
        old = Backend("v1")
        self.lab.install(old)
        before = self.lab.current()["generation"]
        result = self.lab.install(Backend("bad-after-switch", fail_after_checks=1))
        self.assertTrue(result["rolled_back"])
        self.assertFalse(old.stopped)
        self.assertEqual(self.get()["version"], "v1")
        self.assertGreater(self.lab.current()["generation"], before)

    def test_stale_route_compare_and_swap_rejected(self):
        self.lab.install(Backend("v1"))
        stale = self.lab.current_raw()
        self.lab.install(Backend("v2"))
        candidate = Backend("stale-v3")
        self.lab.backends[candidate.id] = candidate
        with self.assertRaises(RedisError):
            self.lab.compare_switch(stale, candidate)
        self.assertEqual(self.lab.current()["version"], "v2")

    def test_expired_owner_cannot_switch(self):
        self.lab.install(Backend("v1"))
        candidate = Backend("v2")
        self.lab.backends[candidate.id] = candidate
        self.redis.command("SET", self.lab.lease, "new-owner", "PX", 5000)
        result = self.redis.command("EVAL", SWITCH, 4, self.lab.lease, self.lab.route_key,
                                    self.lab.generation, self.lab.lifecycle, "expired-owner", self.lab.current_raw(),
                                    json.dumps(candidate.route()))
        self.assertEqual(result, [0, "lease"])
        self.assertEqual(self.lab.current()["version"], "v1")

    def test_retired_backend_cannot_be_reactivated(self):
        old = Backend("v1")
        self.lab.install(old)
        self.lab.install(Backend("v2"))
        self.assertTrue(old.stopped)
        with self.assertRaises(RedisError):
            self.lab.compare_switch(self.lab.current_raw(), old)
        self.assertEqual(self.lab.current()["version"], "v2")

    def test_busy_old_backend_retained_after_drain_deadline(self):
        old = Backend("v1", delay=0.5)
        self.lab.install(old)
        with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
            request = pool.submit(self.get)
            deadline = time.monotonic() + 3
            while self.lab.outstanding(old.id) == 0 and time.monotonic() < deadline:
                time.sleep(0.005)
            result = self.lab.install(Backend("v2"), drain_timeout=0.01)
            self.assertFalse(result["old_drained"])
            self.assertFalse(old.stopped)
            self.assertEqual(request.result()["version"], "v1")
        self.assertTrue(self.lab.drain(old))
        self.assertTrue(old.stopped)

    def test_full_lab_report(self):
        result = exercise({"redis_port": self.port, "requests": 80})
        self.assertTrue(result["ok"])
        self.assertEqual(result["completed"], 80)
        self.assertEqual(result["failures"], 0)
        self.assertTrue(result["unhealthy_rejected"])
        self.assertTrue(result["post_activation_rollback"])


class ConfigTests(unittest.TestCase):
    def test_remote_redis_rejected(self):
        with self.assertRaises(ValueError):
            Redis("192.0.2.1")


if __name__ == "__main__":
    unittest.main()
