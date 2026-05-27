from __future__ import annotations

import argparse
import asyncio
import math
import statistics
import time
from dataclasses import dataclass

from bochat_sdk import ApiError, BochatClient, CreateGroupRequest
from bochat_sdk.error import HttpStatusError, TransportError


@dataclass
class RampStepResult:
    target_rps: int
    total: int
    success: int
    failed: int
    p95_ms: float
    p99_ms: float
    avg_ms: float
    elapsed_s: float

    @property
    def fail_rate(self) -> float:
        if self.total == 0:
            return 0.0
        return self.failed / self.total


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="BoChat message send ramp test: 10 msg/s exponential growth until 1000 or degradation."
    )
    parser.add_argument("--base-url", default="http://127.0.0.1:8080")
    parser.add_argument("--start-rps", type=int, default=10)
    parser.add_argument("--max-rps", type=int, default=100)
    parser.add_argument("--growth", type=float, default=2.0)
    parser.add_argument("--step-seconds", type=int, default=15)
    parser.add_argument("--fail-rate-threshold", type=float, default=0.05)
    parser.add_argument("--p95-ms-threshold", type=float, default=800.0)
    parser.add_argument("--account-prefix", default="perf_user")
    parser.add_argument("--password", default="Passw0rd!")
    parser.add_argument("--group-prefix", default="PERF")
    return parser.parse_args()


async def ensure_user_and_login(client: BochatClient, account: str, password: str) -> None:
    try:
        await (
            client.auth()
            .register()
            .account(account)
            .password(password)
            .nickname(account)
            .send()
        )
    except ApiError as err:
        if err.code != "account_conflict":
            raise
        await client.auth().login().account(account).password(password).send()


async def create_test_group(client: BochatClient, prefix: str) -> tuple[str, str]:
    bots = await client.bots().list()
    if not bots:
        raise RuntimeError("No available bot for the test account.")
    bot = bots[0]
    client.set_bot_token(bot.token)

    ts = int(time.time())
    group_code = f"{prefix}{ts % 1000000}"
    group = await client.groups().create(
        CreateGroupRequest(
            name=f"perf-group-{ts}",
            description="message ramp perf test",
            group_code=group_code,
            bot_id=bot.bot_id,
        )
    )
    return group.group_id, bot.bot_id


async def send_one(client: BochatClient, group_id: str, index: int) -> tuple[bool, float]:
    start = time.perf_counter()
    try:
        await client.messages().send_text(group_id, f"perf msg #{index}")
        return True, (time.perf_counter() - start) * 1000.0
    except (ApiError, HttpStatusError, TransportError):
        return False, (time.perf_counter() - start) * 1000.0


async def run_step(
    client: BochatClient, group_id: str, target_rps: int, step_seconds: int
) -> RampStepResult:
    print(
        "step_begin",
        f"target_rps={target_rps}",
        f"step_seconds={step_seconds}",
        flush=True,
    )
    interval = 1.0 / float(target_rps)
    total = target_rps * step_seconds
    success = 0
    failed = 0
    latencies_ms: list[float] = []
    tasks: list[asyncio.Task[tuple[bool, float]]] = []

    started = time.perf_counter()
    next_emit = started

    for i in range(total):
        now = time.perf_counter()
        sleep_for = next_emit - now
        if sleep_for > 0:
            await asyncio.sleep(sleep_for)
        tasks.append(asyncio.create_task(send_one(client, group_id, i)))
        next_emit += interval

    for task in asyncio.as_completed(tasks):
        ok, latency_ms = await task
        latencies_ms.append(latency_ms)
        if ok:
            success += 1
        else:
            failed += 1

    elapsed_s = time.perf_counter() - started
    if latencies_ms:
        p95_ms = statistics.quantiles(latencies_ms, n=20)[18]
        p99_ms = statistics.quantiles(latencies_ms, n=100)[98]
        avg_ms = statistics.mean(latencies_ms)
    else:
        p95_ms = 0.0
        p99_ms = 0.0
        avg_ms = 0.0

    return RampStepResult(
        target_rps=target_rps,
        total=total,
        success=success,
        failed=failed,
        p95_ms=p95_ms,
        p99_ms=p99_ms,
        avg_ms=avg_ms,
        elapsed_s=elapsed_s,
    )


def should_stop(
    result: RampStepResult, fail_rate_threshold: float, p95_ms_threshold: float
) -> bool:
    return result.fail_rate > fail_rate_threshold or result.p95_ms > p95_ms_threshold


def next_rps(current: int, growth: float, max_rps: int) -> int:
    stepped = int(math.ceil(current * growth))
    if stepped <= current:
        stepped = current + 1
    return min(stepped, max_rps)


async def main() -> None:
    args = parse_args()
    ts = int(time.time())
    account = f"{args.account_prefix}_{ts}"
    client = BochatClient.builder(args.base_url).timeout_secs(20).build()

    group_id: str | None = None
    try:
        await ensure_user_and_login(client, account, args.password)
        group_id, _ = await create_test_group(client, args.group_prefix)

        print(
            "ramp_start",
            f"base_url={args.base_url}",
            f"group_id={group_id}",
            f"start_rps={args.start_rps}",
            f"max_rps={args.max_rps}",
            f"growth={args.growth}",
            f"step_seconds={args.step_seconds}",
            f"fail_rate_threshold={args.fail_rate_threshold}",
            f"p95_ms_threshold={args.p95_ms_threshold}",
            flush=True,
        )

        rps = args.start_rps
        while True:
            result = await run_step(client, group_id, rps, args.step_seconds)
            print(
                "ramp_step",
                f"target_rps={result.target_rps}",
                f"total={result.total}",
                f"success={result.success}",
                f"failed={result.failed}",
                f"fail_rate={result.fail_rate:.3f}",
                f"avg_ms={result.avg_ms:.2f}",
                f"p95_ms={result.p95_ms:.2f}",
                f"p99_ms={result.p99_ms:.2f}",
                f"elapsed_s={result.elapsed_s:.2f}",
                flush=True,
            )

            if rps >= args.max_rps:
                print("stop_reason reached_max_rps", flush=True)
                break
            if should_stop(result, args.fail_rate_threshold, args.p95_ms_threshold):
                print("stop_reason performance_degradation", flush=True)
                break
            rps = next_rps(rps, args.growth, args.max_rps)
    finally:
        if group_id is not None:
            try:
                await client.groups().delete(group_id)
            except Exception:
                pass
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
