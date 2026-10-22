#!/usr/bin/env python3
"""Linux CLI end-to-end test with real executables, HTTP center and failure injection.

All accounts, keys, secrets and signed fixtures are temporary and synthetic.
This test rebuilds two CLI versions using GESA_RELEASE_VERSION, then restores
the ordinary build. Use a dedicated checkout; do not run concurrently with Cargo.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import time

from demo_center import DemoCenter, _write, _json


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def evidence(binary):
    return {
        "candidate_id": "e2e-synthetic-candidate", "source_revision": "a" * 40,
        "artifact_sha256": digest(binary), "change_summary": "Synthetic protocol fixture",
        "risk": "low", "tests": [{"name": "fixture-health", "passed": True,
                                     "evidence_sha256": hashlib.sha256(b"synthetic-test-evidence").hexdigest()}],
        "approval": {"decision": "approved", "actor": "e2e-policy", "policy": "automatic-low-risk"},
        "rollback": "Restore retained executable; no data changes",
        "migration": {"kind": "none", "reversible": True}, "protected_core_changed": False,
    }


def configure(home, origin, public_key, password="synthetic-password"):
    home.mkdir(mode=0o700, exist_ok=True)
    config = {"schema": 1, "center_url": origin, "account": "synthetic-account",
              "release_public_key": public_key, "channel": "stable",
              "allow_http_loopback": True, "credential_store": "file"}
    _write(home / "config.json", _json(config))
    binding = hashlib.sha256(f"{origin}\nsynthetic-account\n{public_key}\nstable".encode()).hexdigest()
    record = hashlib.sha256(f"{home.resolve()}\n{binding}\ncredentials".encode()).hexdigest()
    _write(home / f"credentials-{record}.secret", _json({"password": password, "passphrase": "synthetic-passphrase"}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin", default="target/debug/gesa")
    parser.add_argument("--launcher", default="target/debug/gesa-launcher")
    parser.add_argument("--report")
    args = parser.parse_args()
    if os.name != "posix":
        raise SystemExit("This process-lifetime harness requires Linux; Windows is covered by Rust CI")
    repo = Path(__file__).resolve().parents[1]
    binary, launcher = Path(args.bin).resolve(), Path(args.launcher).resolve()
    cargo = shutil.which("cargo") or str(Path.home() / ".cargo/bin/cargo")
    rustc = shutil.which("rustc") or str(Path.home() / ".cargo/bin/rustc")
    build_env = os.environ.copy()
    build_env.pop("GESA_RELEASE_VERSION", None)
    results = []

    with tempfile.TemporaryDirectory(prefix="gesa-e2e-") as tmp:
        root = Path(tmp)
        dist = root / "dist"
        dist.mkdir()
        core = dist / "gesa"
        shutil.copy2(binary, core)
        shutil.copy2(launcher, dist / "gesa-launcher")
        candidates = []
        try:
            for version in ("0.1.1", "0.1.2"):
                env = dict(build_env, GESA_RELEASE_VERSION=version)
                subprocess.run([cargo, "build", "--locked", "--bin", "gesa"], cwd=repo, env=env, check=True, stdout=subprocess.DEVNULL)
                dest = root / f"core-{version}"
                shutil.copy2(repo / "target/debug/gesa", dest)
                candidates.append(dest)
        finally:
            subprocess.run([cargo, "build", "--locked"], cwd=repo, env=build_env, check=True, stdout=subprocess.DEVNULL)
        bad_source = root / "unhealthy.rs"
        bad_source.write_text('fn main() { println!("{}", r#"{"healthy":false,"protocol":1,"version":"0.1.3","target":"linux-x86_64"}"#); }')
        bad = root / "unhealthy"
        subprocess.run([rustc, str(bad_source), "-o", str(bad)], check=True)
        center = DemoCenter.init(root / "center", "synthetic-account", "synthetic-password", "synthetic-passphrase")
        server = center.http_server()
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        origin = f"http://127.0.0.1:{server.server_port}"
        home = root / "client"
        configure(home, origin, center.public_key)
        env = dict(os.environ, GESA_HOME=str(home))
        watcher = None

        def run(*arguments, executable=core, ok=True):
            result = subprocess.run([str(executable), *arguments], env=env, text=True, capture_output=True, timeout=25)
            assert (result.returncode == 0) == ok, (arguments, result.returncode, result.stdout, result.stderr)
            for secret in ("synthetic-password", "synthetic-passphrase"):
                assert secret not in result.stdout + result.stderr
            data = json.loads(result.stdout if ok else result.stderr.strip().splitlines()[-1])
            return data.get("result", data)

        def record(name):
            results.append({"name": name, "passed": True})
            print(json.dumps(results[-1]), flush=True)

        state_path = home / "update-state.json"
        release_path = center._release_path("stable", "linux-x86_64")

        def state():
            return json.loads(state_path.read_bytes())

        def signed_variant(envelope, **changes):
            payload = json.loads(base64.b64decode(envelope["payload"]))
            payload.update(changes)
            raw = _json(payload)
            return {"payload": base64.b64encode(raw).decode(), "signature": center._signing_key().sign(raw).hex()}

        try:
            run("install")
            run("install")  # repair/reinstall independent launcher atomically
            installed = home / "bin/gesa"
            assert run("__health", executable=installed)["version"] == "0.1.0"
            assert run("login")["authenticated"]
            record("external-config-hidden-secrets-and-independent-launcher")

            first = center.publish(candidates[0], 1, "0.1.1", "linux-x86_64", evidence(candidates[0]))
            assert run("check")["available"]
            watcher = subprocess.Popen([str(installed), "watch", "--interval", "5"], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            # A 25-second deadline prevents a broken watch command hanging CI.
            import selectors
            selector = selectors.DefaultSelector()
            selector.register(watcher.stdout, selectors.EVENT_READ)
            assert selector.select(25), "watch did not complete its first update"
            first_tick=json.loads(watcher.stdout.readline())
            assert first_tick["ok"] and first_tick["version"]=="0.1.0"
            selector.close()
            assert run("__health", executable=installed)["version"] == "0.1.1"
            assert watcher.poll() is None
            record("signed-real-core-update-and-new-invocation-switch")

            center._sessions.clear()  # Center restart invalidates the cached session.
            assert run("check")["available"] is False
            record("revoked-session-reauthenticates-once")

            second = center.publish(candidates[1], 2, "0.1.2", "linux-x86_64", evidence(candidates[1]))
            assert run("update")["updated"]
            assert run("__health", executable=installed)["version"] == "0.1.2"
            assert watcher.poll() is None
            selector=selectors.DefaultSelector()
            selector.register(watcher.stdout, selectors.EVENT_READ)
            assert selector.select(16), "old process did not complete its next poll"
            next_tick=json.loads(watcher.stdout.readline())
            selector.close()
            assert next_tick["ok"] and next_tick["version"]=="0.1.0"
            assert run("update")["updated"] is False
            assert run("verify")["verified"]
            record("second-update-idempotency-integrity-and-running-old-process")
            watcher.terminate()
            watcher.wait(timeout=5)
            watcher = None

            run("rollback")
            assert run("__health", executable=installed)["version"] == "0.1.1"
            retry = subprocess.run([str(core), "watch", "--once"], env=env, capture_output=True, text=True, timeout=25)
            assert state()["active"]["version"] == "0.1.1"
            assert state()["highest_sequence"] == 2
            record("manual-rollback-survives-next-watch-and-keeps-high-watermark")

            corrupted = dict(second, signature="00" * 64)
            _write(release_path, _json(corrupted))
            run("check", ok=False)
            _write(release_path, _json(first))
            run("check", ok=False)
            for change in ({"expires_at": 1}, {"target": "windows-x86_64"}, {"artifact": "//evil.invalid/payload"}, {"contract_epoch": 4, "supports_contracts": [4]}, {"sequence": 2, "version": "0.1.9"}):
                _write(release_path, _json(signed_variant(second, **change)))
                run("check", ok=False)
            _write(release_path, _json(second))
            record("rejects-signature-replay-expiry-target-origin-contract-and-sequence-attacks")

            third = center.publish(bad, 3, "0.1.3", "linux-x86_64", evidence(bad))
            run("update", ok=False)
            assert state()["active"]["version"] == "0.1.1"
            assert run("__health", executable=installed)["version"] == "0.1.1"
            run("update", ok=False)
            record("failed-health-keeps-previous-core-and-quarantines-candidate")

            # Reproduce the exact durable journal boundary of an interrupted activation.
            current = state()
            payload = json.loads(base64.b64decode(third["payload"]))
            candidate = {key: payload[key] for key in ("sha256", "size", "sequence", "version", "contract_epoch")}
            candidate["bootstrap"] = False
            current["pending"] = {"from": current["active"], "candidate": candidate, "phase": "activated"}
            current["active"] = candidate
            _write(state_path, _json(current))
            assert run("__health", executable=installed)["version"] == "0.1.1"
            assert state()["pending"] is None and state()["highest_sequence"] == 3
            record("interrupted-activation-journal-recovers-offline")

            active = state()["active"]
            damaged = home / "releases" / active["sha256"] / "gesa-core"
            damaged.write_bytes(b"damaged executable")
            assert run("__health", executable=installed)["version"] == "0.1.0"
            assert state()["highest_sequence"] == 3
            record("corrupt-active-core-falls-back-to-independent-bootstrap")

            configure(home, origin, center.public_key, password="incorrect-synthetic-password")
            run("logout")
            run("login", ok=False)
            record("wrong-stored-credential-fails-without-secret-output")
        finally:
            if watcher is not None:
                watcher.terminate()
                watcher.wait(timeout=5)
            server.shutdown()
            server.server_close()
            thread.join(timeout=5)

    report = {"schema": 1, "suite": "gesa-cli-e2e", "platform": "linux", "passed": len(results), "failed": 0, "checks": results}
    if args.report:
        Path(args.report).write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
